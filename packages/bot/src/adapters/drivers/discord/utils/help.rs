use std::sync::LazyLock;

pub static HELP: LazyLock<String> = LazyLock::new(|| {
    let server_url =
        std::env::var("DISCORD_SERVER_URL").unwrap_or_else(|_| String::from("Not available"));

    format!(
        "
 ```ansi
\x1b[1;10;4;31mThe Guessing Game:\x1b[0m
To play the guessing game use the \x1b[1;34m/play\x1b[0m command. This has two options: the set you want to pull the card from; and the difficulty (Easy, Medium, or Hard) this defaults to Medium.

To guess a card use the \x1b[1;34m/guess\x1b[0m command and the bot will tell you if you are correct or not (slight typos are forgiven so no need to be super accurate with spelling).

The more you get the card wrong the more of the card is revealed.


\x1b[1;10;4;31mSearching for cards:\x1b[0m
You can use the \x1b[1;34m/search\x1b[0m command to search a card or do the following.

To search a card simply put your desired card in double square brackets: \x1b[1;34m[[lightning bolt]]\x1b[0m
Don't worry about slight misspellings as the bot with try and find the best match for what you have put

To refine your search you can specify artist/set (both set abbreviation and full name): \x1b[1;34m[[lightning bolt | set=m11]]\x1b[0m or \x1b[1;34m[[relentless rats | artist = thomas m baxa]]\x1b[0m

Additionally, you can put this mid-sentence as to not break your flow, as well as multiple in one message. For example:
\x1b[1;34mI really love the artwork of [[the gitrog monster | set=bloomburrow commander]] it shows his true chonk, the classic [[gitrog monster | set=soi]] is not as cool.\x1b[0m


\x1b[1;10;4;31mAll Commands:\x1b[0m
\x1b[1;34m/search\x1b[0m - Options: (set, artist) - Fuzzy search for the specified Magic the Gathering Card.
\x1b[1;34m/help\x1b[0m - Options: () - Show this message.
\x1b[1;34m/play\x1b[0m - Options: (set, difficulty) - Start a game of guess the Magic the Gathering card.
\x1b[1;34m/guess\x1b[0m - Options: () - Make a guess for an active guess the card game.
\x1b[1;34m/give_up\x1b[0m - Options: () - Give up on the current game and return the answer.

\x1b[1;10;4;31mHaving issues or have suggestions?\x1b[0m
Please raise a ticket here https://github.com/i-h-bar/rustcord/issues

or if you don't want to use github please raise a ticket in this server
{server_url}
```
    "
    )
});