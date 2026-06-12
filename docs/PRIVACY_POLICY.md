# Privacy Policy

_Last updated: 12 June 2026_

## Overview

Rustcord is a Magic: The Gathering Discord bot. This policy describes what data the bot collects, how it is used, and how long it is retained.

## Data Collected

### Message Content
Rustcord requests the Message Content privileged intent solely to support the `[[card name]]` inline search syntax. When a message is received, the bot scans it for the `[[...]]` pattern using a regex. Only the text inside the brackets is extracted and used as a one-time card search query. The remainder of the message is immediately discarded and never accessed.

No message content outside of the `[[...]]` delimiters is read, stored, or processed in any form. The extracted search term is not stored in any database. It may appear in operational logs for debugging purposes only and is not persisted beyond those logs.

### Game State
When a guessing game is started with `/play`, the bot stores game state (the mystery card, difficulty level, and guess count) in a temporary Redis cache keyed by Discord channel ID. This data:
- Contains no personally identifiable information
- Is scoped to the channel, not to any individual user
- Expires automatically after 24 hours
- Is deleted immediately when the game ends (win, loss, or `/give_up`)

### Operational Logs
The bot writes operational logs for debugging purposes. These logs may contain:
- Discord channel IDs
- Card search terms extracted from `[[...]]` queries
- Internal timing and error information

Logs do not contain user IDs, usernames, message content outside `[[...]]`, or any other personal data.

## Data Not Collected

Rustcord does not collect, store, or process:
- User IDs or usernames
- Message content outside of `[[...]]` delimiters
- Direct messages
- Guild membership or role information
- Any data used for advertising, analytics, or machine learning

## Third-Party Services

Rustcord does not share any data with third parties. Card data is served from a self-hosted PostgreSQL database populated from [Scryfall](https://scryfall.com) bulk data.

## Data Retention

Game state in Redis expires after 24 hours. Operational logs are retained at the discretion of the bot operator and are used solely for debugging.

## Contact

For questions or concerns about this privacy policy, please open an issue at [github.com/i-h-bar/rustcord/issues](https://github.com/i-h-bar/rustcord/issues) or join the [Discord server](https://discord.gg/m9FjpPRAxt).