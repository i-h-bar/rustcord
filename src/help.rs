pub const HELP: &str =
    "
 ```ansi
To search a card simple put your desired card in double square brackets: \u{001b}[1;31m[[lightning bolt]]\u{001b}[0m\n\
Don't worry about slight misspellings as the bot with try and find the best match for what you have put\n\n
To refine your search you can specify artist/set (both set abbreviation and full name): \u{001b}[1;31m[[lightning bolt | set=m11]]\u{001b}[0m or \u{001b}[1;34m[[relentless rats | artist = thomas m baxa]]\u{001b}[0m\n
These can be combined in one query.\n\n
Additionally, you can put this mid-sentence as to not break your flow, as well as multiple in one message. For example:\n
\u{001b}[1;32mI really love the artwork of [[the gitrog monster | set=bloomburrow commander]] it shows his true chonk, the classic [[gitrog monster | set=soi]] is not as cool.\u{001b}[0m\n\n
```
    ";
