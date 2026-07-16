
# Rustcord — Magic: The Gathering Discord Bot

![Tests](https://img.shields.io/badge/tests-121%20passing-brightgreen)
![Build](https://github.com/i-h-bar/rustcord/workflows/PR%20Checks/badge.svg)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

A Discord bot for searching Magic: The Gathering cards and playing a card-guessing game. Uses fuzzy matching to handle misspellings and supports searching by name, set, and artist.

**[Invite the bot](https://discord.com/oauth2/authorize?client_id=1315969494161559595&permissions=277025507328&integration_type=0&scope=bot)** · **[Report an issue](https://github.com/i-h-bar/rustcord/issues)** · **[Privacy Policy](docs/PRIVACY_POLICY.md)** · **[Terms of Service](docs/TERMS_OF_SERVICE.md)**

---

## Invite Options

Two invite links are available, depending on whether you want spoiler notifications:

- **[Card search & games only](https://discord.com/oauth2/authorize?client_id=1315969494161559595&permissions=277025507328&integration_type=0&scope=bot)** — the core bot, no extra permissions
- **[Full functionality, incl. spoiler notifications](https://discord.com/oauth2/authorize?client_id=1315969494161559595&permissions=277562378240&integration_type=0&scope=bot)** — adds `Manage Webhooks` so `/spoilers` can post to a channel

If you invited the bot without `Manage Webhooks` and later want spoiler notifications, just use the second link — Discord will prompt to grant the extra permission without needing to kick and re-invite the bot.

---

## Features

- **Card search** via `/search` or inline `[[card name]]` syntax in any message
- **Fuzzy matching** — slight misspellings are forgiven
- **Scoped search** by set name, set code, or artist
- **Printings dropdown** — browse every printing of a card with set symbols
- **Similar cards dropdown** — surfaces close matches if the wrong card was returned
- **Guessing game** with three difficulty levels and progressive clue reveals
- **Spoiler notifications** *(beta)* — auto-post newly spoiled cards to a channel of your choice

---

## Card Search

Use the `/search` command or wrap a card name in double square brackets anywhere in a message:

```
[[lightning bolt]]
```

Refine by set or artist:

```
[[lightning bolt | set=m11]]
[[relentless rats | artist=thomas m baxa]]
[[gitrog monster | set=shadows over innistrad]]
```

You can use inline queries mid-sentence and stack multiple in one message:

```
I really love [[the gitrog monster | set=bloomburrow commander]], the classic [[gitrog monster | set=soi]] is not as cool.
```

Results include a **Select a print** dropdown to browse alternate printings and a **Similar cards** dropdown to navigate to related cards.

---

## Guessing Game

Start a game with `/play`. Options:
- **Set** — limit the mystery card to a specific set
- **Difficulty** — Easy (8 guesses), Medium (6 guesses, default), Hard (4 guesses)

The bot progressively reveals clues — mana cost, type line, rules text, and eventually a cropped illustration. Submit guesses with `/guess` (fuzzy matching applies). Give up with `/give_up` to reveal the answer.

---

## Spoiler Notifications *(beta)*

Get new Magic: The Gathering cards posted automatically to a channel as soon as they're spoiled — no more refreshing spoiler sites.

- `/spoilers subscribe channel:#channel` — start posting new-card spoilers to the given channel
- `/spoilers unsubscribe channel:#channel` — stop posting spoilers to that channel

A server can subscribe as many channels as it likes, each tracked independently. Both commands require **Manage Server** permission to run, and the bot needs **Manage Webhooks** to post — if it's missing that permission, it'll reply with a link to re-authorize it without needing to kick and re-invite the bot.

This feature is still in beta — if something looks wrong, please [report it](https://github.com/i-h-bar/rustcord/issues).

---

## Commands

| Command                 | Options                 | Description                                            |
|-------------------------|-------------------------|---------------------------------------------------------|
| `/search`               | `name`, `set`, `artist` | Fuzzy search for a card                                |
| `/play`                 | `set`, `difficulty`     | Start a guessing game                                  |
| `/guess`                | `card`                  | Submit a guess for the active game                     |
| `/give_up`              | -                       | Reveal the answer and end the game                     |
| `/spoilers subscribe`   | `channel`               | *(Beta)* Start posting new-card spoilers to a channel  |
| `/spoilers unsubscribe` | `channel`               | Stop posting spoilers to a channel                     |
| `/help`                 | -                       | Show command reference                                 |

---

## Demo

![demo](README_images/demo.gif)