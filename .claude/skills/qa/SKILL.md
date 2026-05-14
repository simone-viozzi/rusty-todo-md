---
name: qa
description: Run an iterative QA loop to bridge the gap between Claude's understanding and the user's understanding of a topic. Each iteration is one focused question; the conversation is logged to a durable file written incrementally. Use when the user invokes /qa or says they want to do QA on a topic.
---

# QA workflow

Iterative question-and-answer loop that produces a durable record bridging your understanding of a topic to the user's. Used for context gathering, alignment, and decision-shaping.

## Trigger

Invoked by `/qa` or when the user says they want to do QA on a topic.

## Startup

1. **If prior conversation context exists**: propose a topic + objective drawn from it. Wait for confirmation/edit before proceeding.
2. **If no prior context exists**: run a small meta-loop (using the same RE/R/Q/A/RA format below, without writing to file) to nail down the objective itself, then proceed.

## File location

**Propose a path and wait for the user's confirmation before writing.**

- Heuristic: place the file **near related notes** when the topic clearly maps to an existing area (e.g., a QA about something in `index/work/foo/` lands as `index/work/foo/QA-<timestamp>-<topic-slug>.md`).
- Do **not** create a standalone `qa/` folder.
- If no clear home exists, ask the user where to save it.

## Filename

`QA-YYYY-MM-DDTHHMM-topic-slug.md` — full timestamp including time (not just date), slugified topic.

## File contents

### Intro (written once at file creation)

```markdown
# QA: <topic>

**Objective:** <what this QA is for, agreed with the user>
**Started:** <full timestamp>
```

### Iteration log (one block appended per turn)

After each answer, **write the file imperatively** — append the completed block to the file *before* forming the next question. The file should always reflect the conversation up to the latest answered question.

Block format:

```markdown
## Q<n>

**RE** — <research notes; only include if you actually researched for context, otherwise omit this line>

**R** — <reasoning on why this question matters now>

**Q** — <one precise, focused question>

**A** — <user's answer, verbatim>

**RA** — <reasoning on the answer: what it tells you, what gap it closed, what new gap it opened>
```

## Loop discipline

- **One question at a time.** Always. No multi-question turns, no "and also" follow-ups in the same Q.
- After each Q&A, **write the iteration block to the file immediately**, then form the next question.
- Throughout the loop, track internally "what have I actually learned" so you're ready to write the synthesis on demand.

## Termination

- Propose **"I think I have enough — want to wrap?"** when you sense diminishing returns.
- The user can also stop you at any turn.

## Wrap-up

When the user confirms wrap:

1. Append an **"Understanding so far"** synthesis section to the file, consolidating what you learned across the loop.
2. Present it to the user for review.
3. **If the user flags discrepancies → scrap the synthesis section** (it means you didn't actually understand) and continue the loop with new questions targeting the gaps. Re-run wrap-up later.
4. If confirmed → done.

## Language

Match the language the user is using in the conversation. Don't impose a default.
