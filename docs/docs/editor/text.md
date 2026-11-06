# Text format
A readable text format for conversations. Primary goal is to make it easy to
write conversations that flow better, but has the additonal benefit of making it
possible to eventually phase out the editor, and to make the tool more familiar
to users coming from other text based systems.

# Comments
Comments are started by a `%` and remove all text between them and the end of the
line from the view of the parser. If a line starts with %

# Hubs
Hubs are separated by 2 or more newlines plus any amount of whitespace. They
contain "unlabeled" entries, "labeled" entries, and choices separated by newlines.

```
tags: hub_1 labeled_entry
another_labeled_entry: some text
Speaker for hub 1 in first unlabeled entry
Text for hub 1 in second unlabeled entry
More text for hub 1
> hub_3

tags: hub_2
I'm unreachable! Also, since this is the only unlabeled entry, it's interpreted as text instead of a speaker.

tags: hub_3
Text only again. This text is reachable from hub_1. 
```

And that's it! Entries with labels matching those in [schema](./schema.md) will
behave similarly.