# Rewrite Records

Rewrite records capture non-obvious lessons, source-boundary decisions and reusable migration insights. They are inspired by the `teach` skill's learning records, but they are engineering records, not lessons.

## Naming

Use:

```text
rewrite-records/0001-<dash-case-name>.md
```

Increment the number for each new record.

## Required Sections

- Context
- Decision or Lesson
- Applies To
- Does Not Imply
- Follow-up

## When To Add One

- A source-borrowing boundary changes, such as what is or is not borrowed from py2rs.
- A migration batch reveals a lesson that should affect future batches.
- A quality gate catches a class of issue likely to recur.
- A user preference changes how future agents should work.
