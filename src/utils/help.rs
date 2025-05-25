pub const HELP: &str =
    "
 ```ansi
\u{001b}[1;10;4;31mThe Guessing Game:\u{001b}[0m
To play the guessing game use the \u{001b}[1;34m/play\u{001b}[0m command. This has two options: the set you want to pull the card from; and the difficulty (Easy, Medium, or Hard) this defaults to Medium.

To guess a card use the \u{001b}[1;34m/guess\u{001b}[0m command and the bot will tell you if you are correct or not (slight typos are forgiven so no need to be super accurate with spelling).

The more you get the card wrong the more of the card is revealed.


\u{001b}[1;10;4;31mSearching for cards:\u{001b}[0m
You can use the \u{001b}[1;34m/search\u{001b}[0m command to search a card or do the following.

To search a card simply put your desired card in double square brackets: \u{001b}[1;34m[[lightning bolt]]\u{001b}[0m
Don't worry about slight misspellings as the bot with try and find the best match for what you have put

To refine your search you can specify artist/set (both set abbreviation and full name): \u{001b}[1;34m[[lightning bolt | set=m11]]\u{001b}[0m or \u{001b}[1;34m[[relentless rats | artist = thomas m baxa]]\u{001b}[0m

Additionally, you can put this mid-sentence as to not break your flow, as well as multiple in one message. For example:
\u{001b}[1;34mI really love the artwork of [[the gitrog monster | set=bloomburrow commander]] it shows his true chonk, the classic [[gitrog monster | set=soi]] is not as cool.\u{001b}[0m


\u{001b}[1;10;4;31mAll Commands:\u{001b}[0m
\u{001b}[1;34m/search\u{001b}[0m - Options: (set, artist) - Fuzzy search for the specified Magic the Gathering Card.
\u{001b}[1;34m/help\u{001b}[0m - Options: () - Show this message.
\u{001b}[1;34m/play\u{001b}[0m - Options: (set, difficulty) - Start a game of guess the Magic the Gathering card.
\u{001b}[1;34m/guess\u{001b}[0m - Options: () - Make a guess for an active guess the card game.


\u{001b}[1;10;4;31mHaving issues or have suggestions?\u{001b}[0m
Please raise a ticket here https://github.com/i-h-bar/rustcord/issues

of if you don't want to use github please raise a ticket in this server
https://discord.gg/m9FjpPRAxt
```
    ";
