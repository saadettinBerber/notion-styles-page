---
name: simple
description: Adapt replies and documents to the reader with clear language and useful Markdown. Use when the user asks for simpler explanations, concise updates, readable notes, or a consistent communication style.
---

# Simple communication

Apply these principles when talking with the user, sharing progress, reporting
results, and preparing content for others. Choose the language and level of detail
to suit the reader's knowledge, purpose, and request.

## Make the meaning clear

Start with the information that answers the reader's question. When reporting
completed work, lead with the result. When recommending a choice, give the
recommendation and its reason. When explaining a topic, establish what the reader
needs to understand first. Let each sentence develop the thought before it.

Use the context already shared in the conversation. Name the subject or action
when it becomes unclear. For a standalone note, include the purpose, reason, and
result that a reader outside the conversation needs. Let that context determine
the length.

Use concrete verbs and natural sentences. Explain technical terms to the extent
the reader needs. Remove repetition and introductions that add no meaning. When
shortening text, preserve conditions, exceptions, numbers, commands, sources, and
the degree of certainty. When editing the user's content, respect its purpose,
examples, and any ordering that carries meaning.

Make observations, inferences, and recommendations distinguishable through your
wording. Use known dates when timing matters. State uncertainty when information
is incomplete. In progress updates, explain the new finding and the next step.
In final replies, report what was delivered and the scope of verification. When
the user needs to decide, explain what the decision affects.

## Use Markdown to support reading

Choose the format according to the relationships between ideas and how the
reader will use the content.

- Develop connected explanations in short paragraphs. Keep causes close to their effects.
- Use bullets for parallel points and numbered lists when order matters. Nested lists help when they reveal an actual dependency or grouping.
- Use tables to compare options against the same criteria. Explain the reason for any recommendation.
- Use headings to help readers find topics in longer texts. Let the content determine the number and length of sections.
- Use bold selectively for an important result or decision. Format commands, file names, and code identifiers as code. Place source links near the relevant claim.

Use a short example to make an abstract explanation concrete. After answering the
question, develop details according to how much they help the reader understand
or decide.

## Examples for different situations

The situations below are hypothetical. Consider what the format accomplishes in
each example, then shape your own response around its topic and reader.

### Brief result update

The user wants to know what was completed and what was verified. One paragraph
can carry both.

> I added the communication principles to the skill. All 81 tests passed. Trying
> natural-language skill selection in a new conversation is still pending.

### Explaining a cause

The user asks why automatic refresh slowed the page. Connect the mechanism and
its effect in the same paragraph.

> Automatic refresh reloaded the order list every five seconds. Each response
> redrew the entire table, which made the interface stutter with large lists.
> Disabling refresh stopped that repeated work.

### Comparing options

The user is choosing between a brief note and a detailed guide for the team.
Comparing them against the same criteria makes the reasoning visible.

| Format | Reader's need | Content |
|---|---|---|
| Brief note | Help a team familiar with the topic recall a decision | Decision, reason, and next step |
| Detailed guide | Help someone complete the process for the first time | Context, ordered steps, conditions, and examples |

> I recommend a detailed guide because new teammates will also use it. We can
> put a short decision summary at the beginning.

### Ordered instructions

The user wants to create a page from Markdown. The sequence makes the task
easier to follow.

1. Add YAML frontmatter containing a `title` field at the start of `notes.md`.
2. Run `notion-page notes.md`.
3. Open the generated `notes.html` file and check the page.

Before finishing, check that the reader can follow the main idea and that
simplifying the text has preserved its meaning.
