# Output contract

The Chair MUST produce the report in exactly this shape and order. The point is a
usable decision artifact: the single best call first, the reasoning behind it, then the
alternatives — never an undifferentiated wall of opinions.

```markdown
# Council — <repo type>:<unit> — focus: <lens>

## Best call
<One recommendation: the move with the SMALLEST residual downside.>
- Residual negative value: <what you still lose / risk if you do this>
- Reversibility: <easy | costly | one-way door>
- What would flip this: <the concrete evidence that would change the recommendation>

## Disagreement map
The real tensions (2–4 max). For each: the crux, and who is on each side.
- **<tension>** — <Seat A> says X because …; <Seat B> says Y because …. Crux: <the one fact or value the call hinges on>.

## Recommendations (ranked by leverage)
| # | Move | Leverage | Residual negative | Reversibility | Evidence to flip |
|---|------|----------|-------------------|---------------|------------------|
| 1 | …    | high     | …                 | easy          | …                |
| 2 | …    | med      | …                 | costly        | …                |

## Maturity scorecard (only when focus = maturity)
Score each seated technical seat on ITS OWN axis (from persona `maturity_axis`).
| Seat | Axis | Score (1–5) | One sentence why |
|------|------|-------------|------------------|

## Parking lot
Ideas raised but out of this run's focus — captured for a later council, not acted on now.
- <e.g. "spin up backbone-notify as a shared module"> — raised by <seat>, scope: <root/other>.
```

## Rules for the Chair

- The Best call is one move, not a menu. If you cannot pick, say which single piece of
  evidence is missing and what cheap probe would get it — that probe becomes the Best call.
- Quantify residual negative value concretely (time, coupling, risk surface), not as
  "some downsides".
- Never merge two opposed seats into a fake middle. Pick a side and own it, or name the
  missing fact that prevents picking.
- Keep the whole report bounded to the chosen focus lens. Everything else is parking lot.
