
# Rustcord — Magic: The Gathering Discord Bot

![Tests](https://img.shields.io/badge/tests-121%20passing-brightgreen)
![Build](https://github.com/i-h-bar/rustcord/workflows/PR%20Checks/badge.svg)
[![Discord](https://img.shields.io/discord/1375617050793349222?color=7289da&label=Discord&logo=discord&logoColor=white)](https://discord.gg/m9FjpPRAxt)

A Discord bot for searching Magic: The Gathering cards and playing a card-guessing game. Uses fuzzy matching to handle misspellings and supports searching by name, set, and artist.

**[Invite the bot](https://discord.com/oauth2/authorize?client_id=1315969494161559595&permissions=277025507328&integration_type=0&scope=bot)** · **[Discord server](https://discord.gg/m9FjpPRAxt)** · **[Report an issue](https://github.com/i-h-bar/rustcord/issues)**

---

## Features

- **Card search** via `/search` or inline `[[card name]]` syntax in any message
- **Fuzzy matching** — slight misspellings are forgiven
- **Scoped search** by set name, set code, or artist
- **Printings dropdown** — browse every printing of a card with set symbols
- **Similar cards dropdown** — surfaces close matches if the wrong card was returned
- **Guessing game** with three difficulty levels and progressive clue reveals

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

## Commands

| Command    | Options               | Description                           |
|------------|-----------------------|---------------------------------------|
| `/search`  | `name`, `set`, `artist` | Fuzzy search for a card             |
| `/play`    | `set`, `difficulty`   | Start a guessing game                 |
| `/guess`   | `card`                | Submit a guess for the active game    |
| `/give_up` | -                     | Reveal the answer and end the game    |
| `/help`    | -                     | Show command reference                |

---

## Demo

![demo](README_images/demo.gif)