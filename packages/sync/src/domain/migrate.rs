//! One-off migration of the legacy PNG image library into the `card-images`
//! volume. Reads local files only (never Scryfall) and never writes under
//! `from`. Unlike the rest of the domain this touches the filesystem
//! directly: it is a one-shot batch job, not part of the sync flow.

use crate::domain::utils::webp::to_webp;
use contracts::image::{ImageFile, ImageKind};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

const PROGRESS_EVERY: usize = 1000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub images_converted: usize,
    pub illustrations_copied: usize,
    pub skipped_existing: usize,
    pub failed: usize,
}

enum Outcome {
    Converted,
    Copied,
    Skipped,
    Failed,
}

/// Converts `from/images/*.png` to `to/images/{id}.webp` and copies
/// `from/illustrations/*.png` (really JPEGs) to `to/illustrations/{id}.jpg`.
///
/// Resumable: targets that already exist with content are skipped, and every
/// write goes through a `.tmp` file + rename so a killed run never leaves a
/// partial file that looks finished. Unreadable or undecodable sources are
/// logged and counted as failed; the normal sync re-downloads them later.
pub fn migrate_images(from: &Path, to: &Path) -> MigrationReport {
    let mut jobs = sources(&from.join("images"), ImageFile::card);
    jobs.extend(sources(
        &from.join("illustrations"),
        ImageFile::illustration,
    ));

    for dir in ["images", "illustrations"] {
        if let Err(why) = fs::create_dir_all(to.join(dir)) {
            log::error!("Failed to create {}: {why}", to.join(dir).display());
        }
    }

    let total = jobs.len();
    log::info!(
        "Migrating {total} files from {} to {}",
        from.display(),
        to.display()
    );

    let processed = AtomicUsize::new(0);
    let outcomes: Vec<Outcome> = jobs
        .par_iter()
        .map(|(src, file)| {
            let outcome = migrate_one(src, &target_path(to, *file), file.kind());
            let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if done % PROGRESS_EVERY == 0 {
                log::info!("Migrated {done}/{total} files");
            }
            outcome
        })
        .collect();

    let mut report = MigrationReport::default();
    for outcome in outcomes {
        match outcome {
            Outcome::Converted => report.images_converted += 1,
            Outcome::Copied => report.illustrations_copied += 1,
            Outcome::Skipped => report.skipped_existing += 1,
            Outcome::Failed => report.failed += 1,
        }
    }

    log::info!("Migration finished: {report:?}");
    report
}

fn target_path(to: &Path, file: ImageFile) -> PathBuf {
    let dir = match file.kind() {
        ImageKind::Card => "images",
        ImageKind::Illustration => "illustrations",
    };
    to.join(dir).join(file.to_string())
}

/// `*.png` files in `dir` whose stem is a UUID. Anything else is ignored.
fn sources(dir: &Path, file: fn(Uuid) -> ImageFile) -> Vec<(PathBuf, ImageFile)> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(why) => {
            log::warn!("Skipping {}: {why}", dir.display());
            return Vec::new();
        }
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .filter_map(|path| {
            let id = Uuid::parse_str(path.file_stem()?.to_str()?).ok()?;
            Some((path, file(id)))
        })
        .collect()
}

fn migrate_one(src: &Path, target: &Path, kind: ImageKind) -> Outcome {
    if fs::metadata(target).is_ok_and(|meta| meta.len() > 0) {
        return Outcome::Skipped;
    }

    let bytes = match fs::read(src) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        Ok(_) => {
            log::warn!("Skipping empty source {}", src.display());
            return Outcome::Failed;
        }
        Err(why) => {
            log::warn!("Failed to read {}: {why}", src.display());
            return Outcome::Failed;
        }
    };

    let (out, outcome) = match kind {
        ImageKind::Card => match to_webp(&bytes) {
            Ok(webp) => (webp, Outcome::Converted),
            Err(why) => {
                log::warn!("Failed to convert {}: {why}", src.display());
                return Outcome::Failed;
            }
        },
        ImageKind::Illustration => (bytes, Outcome::Copied),
    };

    let tmp = target.with_extension(format!("{}.tmp", kind.extension()));
    if let Err(why) = fs::write(&tmp, &out).and_then(|()| fs::rename(&tmp, target)) {
        log::warn!("Failed to write {}: {why}", target.display());
        let _ = fs::remove_file(&tmp);
        return Outcome::Failed;
    }

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::utils::webp::{fixture_jpeg, fixture_png};
    use std::collections::BTreeMap;
    use std::fs;
    use uuid::Uuid;

    struct Dirs {
        _tmp: tempfile::TempDir,
        from: PathBuf,
        to: PathBuf,
    }

    fn dirs() -> Dirs {
        let tmp = tempfile::tempdir().unwrap();
        let from = tmp.path().join("from");
        let to = tmp.path().join("to");
        fs::create_dir_all(from.join("images")).unwrap();
        fs::create_dir_all(from.join("illustrations")).unwrap();
        Dirs {
            _tmp: tmp,
            from,
            to,
        }
    }

    fn write(path: PathBuf, bytes: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let bytes = fs::read(&path).unwrap();
                    out.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
                }
            }
        }
        out
    }

    fn starts_with_riff(path: PathBuf) -> bool {
        fs::read(path).unwrap().starts_with(b"RIFF")
    }

    #[test]
    fn converts_images_and_copies_illustrations() {
        let d = dirs();
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        write(d.from.join(format!("images/{a}.png")), &fixture_png());
        write(
            d.from.join(format!("illustrations/{b}.png")),
            b"\xFF\xD8jpeg",
        );

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(
            report,
            MigrationReport {
                images_converted: 1,
                illustrations_copied: 1,
                ..Default::default()
            }
        );
        assert!(starts_with_riff(d.to.join(format!("images/{a}.webp"))));
        assert_eq!(
            fs::read(d.to.join(format!("illustrations/{b}.jpg"))).unwrap(),
            b"\xFF\xD8jpeg"
        );
    }

    #[test]
    fn converts_jpeg_named_png() {
        let d = dirs();
        let a = Uuid::new_v4();
        write(d.from.join(format!("images/{a}.png")), &fixture_jpeg());

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(report.images_converted, 1);
        assert!(starts_with_riff(d.to.join(format!("images/{a}.webp"))));
    }

    #[test]
    fn skips_existing_targets() {
        let d = dirs();
        let a = Uuid::new_v4();
        write(d.from.join(format!("images/{a}.png")), &fixture_png());
        write(d.to.join(format!("images/{a}.webp")), b"keep");

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(report.skipped_existing, 1);
        assert_eq!(report.images_converted, 0);
        assert_eq!(
            fs::read(d.to.join(format!("images/{a}.webp"))).unwrap(),
            b"keep"
        );
    }

    #[test]
    fn resumes_over_tmp_and_empty_targets() {
        let d = dirs();
        let (a, c) = (Uuid::new_v4(), Uuid::new_v4());
        write(d.from.join(format!("images/{a}.png")), &fixture_png());
        write(d.from.join(format!("images/{c}.png")), &fixture_png());
        write(d.to.join(format!("images/{a}.webp")), b"");
        write(d.to.join(format!("images/{c}.webp.tmp")), b"junk");

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(report.images_converted, 2);
        assert!(starts_with_riff(d.to.join(format!("images/{a}.webp"))));
        assert!(starts_with_riff(d.to.join(format!("images/{c}.webp"))));
        let leftovers: Vec<_> = fs::read_dir(d.to.join("images"))
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|x| x == "tmp")
            })
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn skips_empty_and_corrupt_sources() {
        let d = dirs();
        let (a, b, ok) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        write(d.from.join(format!("images/{a}.png")), b"");
        write(d.from.join(format!("images/{b}.png")), b"garbage");
        write(d.from.join(format!("images/{ok}.png")), &fixture_png());

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(report.failed, 2);
        assert_eq!(report.images_converted, 1);
        assert!(!d.to.join(format!("images/{a}.webp")).exists());
        assert!(!d.to.join(format!("images/{b}.webp")).exists());
    }

    #[test]
    fn skips_empty_illustration_sources() {
        let d = dirs();
        let a = Uuid::new_v4();
        write(d.from.join(format!("illustrations/{a}.png")), b"");

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(report.failed, 1);
        assert!(!d.to.join(format!("illustrations/{a}.jpg")).exists());
    }

    #[test]
    fn ignores_sets_and_non_png_files() {
        let d = dirs();
        write(d.from.join("sets/x.png"), &fixture_png());
        write(d.from.join("images/readme.txt"), b"hi");
        write(d.from.join("images/not-a-uuid.png"), &fixture_png());

        let report = migrate_images(&d.from, &d.to);

        assert_eq!(report, MigrationReport::default());
        assert!(!d.to.join("sets").exists());
        assert!(snapshot(&d.to).is_empty());
    }

    #[test]
    fn never_modifies_source() {
        let d = dirs();
        write(
            d.from.join(format!("images/{}.png", Uuid::new_v4())),
            &fixture_png(),
        );
        write(d.from.join(format!("images/{}.png", Uuid::new_v4())), b"");
        write(
            d.from.join(format!("illustrations/{}.png", Uuid::new_v4())),
            b"\xFF\xD8jpeg",
        );
        write(d.from.join("sets/x.png"), b"set");
        let before = snapshot(&d.from);

        migrate_images(&d.from, &d.to);

        assert_eq!(snapshot(&d.from), before);
    }
}
