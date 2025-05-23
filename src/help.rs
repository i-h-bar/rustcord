pub const HELP: &str =
    "
 ```ansi
\u{001b}[1;31mThe Guessing Game:\u{001b}[0m
To play the guessing game use the \u{001b}[1;31m/play\u{001b}[0m command. This has two options: the set you want to pull the card from; and the difficulty (Easy, Medium, or Hard) this defaults to Easy.\n\n
To guess a card use the \u{001b}[1;31m/guess\u{001b}[0m command and the bot will tell you if you are correct or not (slight typos are forgiven so no need to be super accurate with spelling).\n\n\
The more you get the card wrong the more of the card is revealed.\n\n\n
\u{001b}[1;31mSearching for cards:\u{001b}[0m
To search a card simple put your desired card in double square brackets: \u{001b}[1;31m[[lightning bolt]]\u{001b}[0m\n\
Don't worry about slight misspellings as the bot with try and find the best match for what you have put\n\n
To refine your search you can specify artist/set (both set abbreviation and full name): \u{001b}[1;31m[[lightning bolt | set=m11]]\u{001b}[0m or \u{001b}[1;34m[[relentless rats | artist = thomas m baxa]]\u{001b}[0m\n\n
Additionally, you can put this mid-sentence as to not break your flow, as well as multiple in one message. For example:\n
\u{001b}[1;32mI really love the artwork of [[the gitrog monster | set=bloomburrow commander]] it shows his true chonk, the classic [[gitrog monster | set=soi]] is not as cool.\u{001b}[0m\n\n
```
    ";
