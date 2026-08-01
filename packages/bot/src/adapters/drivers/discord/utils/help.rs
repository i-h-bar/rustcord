use std::sync::LazyLock;

pub static HELP: LazyLock<String> = LazyLock::new(|| {
    let server_url =
        std::env::var("DISCORD_SERVER_URL").unwrap_or_else(|_| String::from("Not available"));

    format!(
        "
 ```ansi
\x1b[1;10;4;31mThe Guessing Game:\x1b[0m
Use \x1b[1;34m/play\x1b[0m to start. Options: set (pull the card from a specific set) and difficulty (Easy, Medium, or Hard — defaults to Medium).

Use \x1b[1;34m/guess\x1b[0m to guess — slight typos are forgiven, no need to be super accurate with spelling.

The more you get the card wrong the more of the card is revealed.


\x1b[1;10;4;31mSearching for cards:\x1b[0m
You can use the \x1b[1;34m/search\x1b[0m command to search a card or do the following.

To search a card simply put your desired card in double square brackets and mention the bot: \x1b[1;34m[[lightning bolt]]\x1b[0m — slight misspellings are forgiven.

To refine your search you can specify artist/set (both set abbreviation and full name): \x1b[1;34m[[lightning bolt | set=m11]]\x1b[0m or \x1b[1;34m[[relentless rats | artist = thomas m baxa]]\x1b[0m

You can also use these mid-sentence, and stack several in one message:
\x1b[1;34mI love [[the gitrog monster | set=bloomburrow commander]], the classic [[gitrog monster | set=soi]] is still cool too.\x1b[0m


\x1b[1;10;4;31mAll Commands:\x1b[0m
\x1b[1;34m/search\x1b[0m - Options: (set, artist) - Fuzzy search for the specified Magic the Gathering Card.
\x1b[1;34m/help\x1b[0m - Options: () - Show this message.
\x1b[1;34m/play\x1b[0m - Options: (set, difficulty) - Start a game of guess the Magic the Gathering card.
\x1b[1;34m/guess\x1b[0m - Options: () - Make a guess for an active guess the card game.
\x1b[1;34m/give_up\x1b[0m - Options: () - Give up on the current game and return the answer.
\x1b[1;34m/spoilers\x1b[0m - Options: (subscribe/unsubscribe, channel) - [Beta] Auto-post new cards to a channel.

\x1b[1;10;4;31mHaving issues or have suggestions?\x1b[0m
Please raise a ticket here https://github.com/i-h-bar/rustcord/issues

or if you don't want to use github please raise a ticket in this server
{server_url}
```
    "
    )
});
