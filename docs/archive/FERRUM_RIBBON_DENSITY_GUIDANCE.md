# Ferrum Ribbon Density and Layout Design Guidance

## Purpose

Ferrum should retain the task organization and discoverability of a ribbon while recovering the information density that made the original BKChem interface effective for chemistry drawing.

The current Ferrum ribbon is structurally sound, but it gives too much space to individual commands. Chemistry drawing is unusually well suited to compact controls because many commands have intrinsically recognizable visual symbols: bonds, wedges, rings, arrows, brackets, atom marks, and geometric shapes. Ferrum should use those symbols directly rather than turning most commands into large labeled cards.

This document defines the desired ribbon presentation model. It is primarily a UI layout and presentation contract. It should not move command behavior into the ribbon layer.

---

## Existing architectural direction

Ferrum already has the correct foundation for making this change.

`menus.yaml` defines the canonical menu hierarchy and groups commands by task. QAction construction and behavior remain elsewhere. The current menu resource explicitly describes itself as owning canonical menu clients and task grouping rather than action behavior.

`ribbon_layout.yaml` similarly declares a task-oriented ribbon using the same action IDs. It currently distinguishes `primary` and `supporting` roles and assigns priorities. This is useful metadata, but the renderer currently turns those distinctions into controls that are much larger than Ferrum needs.

The original BKChem configuration also provides useful evidence about the intended density. Its `modes.yaml` describes many related submodes as groups of compact options. Bond order, bond type, atom marks, arrows, transformations, vector shapes, and repair operations were all designed as collections of rapidly selectable tools.

Ferrum should preserve the newer declarative command architecture while recovering that compact interaction model.

---

# Design objective

The ribbon should feel like a **chemistry tool surface organized by tasks**, not a dashboard of command cards.

A user working on a molecule should be able to scan a tab and visually identify many available tools without opening overflow menus. The ribbon should expose substantially more commands in the same horizontal space than the current implementation.

The desired model is closest to:

**BKChem tool density + LibreOffice-style ribbon grouping + Ferrum's declarative command architecture**

The ribbon should remain visibly organized into tabs and named groups. Within those groups, however, commands should be packed into a small number of standardized control sizes.

---

# Core principles

## 1. Prefer tool density over large command cards

Large controls should be exceptional.

A chemistry editor has many commands that users learn visually. A single bond, double bond, wedge, ring, reaction arrow, bracket, rectangle, or atom mark does not need a large text label permanently occupying ribbon space.

Use the icon as the primary representation when the command has a strong visual identity.

Use tooltips, accessible names, status text, and the command palette to provide the full command name.

Text labels should remain visible where the icon alone is genuinely ambiguous.

---

## 2. Use a small discrete vocabulary of control sizes

Ferrum should not size each button according to its label.

Ferrum should also not make every command the same large size.

Use three canonical presentation sizes:

### Compact

Approximately 28 to 32 px square.

Use for:

- bond types
- bond orders
- wedges
- atom marks
- common rings
- arrows with distinctive icons
- brackets
- basic shapes
- alignment controls
- zoom controls
- toggles
- other visually obvious tools

Compact controls are normally icon-only.

A tooltip and accessible name provide the command label.

### Standard

Approximately 70 to 100 px wide and 28 to 32 px high.

Use for commands where a short label materially improves recognition.

Examples:

- Move Atom
- Create Reaction
- Reaction Inspector
- Clean Geometry
- Zoom to Content

A standard control normally places a small icon beside a concise label.

### Large

Approximately 60 to 80 px wide and 56 to 64 px high.

Use sparingly for a dominant task or mode where the larger target and visual emphasis are useful.

Examples might include:

- Select Structure
- Add Atom
- Draw Bond

A tab should generally contain only a few large controls.

`role: primary` must not automatically mean `size: large`.

Role expresses task importance. Size expresses presentation needs. These are separate concepts.

---

# Ribbon geometry

## Target height

Reduce the working ribbon height substantially.

The ribbon should normally fit:

1. the tab strip,
2. two compact command rows,
3. group labels.

The command area should be designed around two compact rows rather than one row of large cards.

A practical target is approximately:

- tab/header area: 36 to 40 px
- command area: 64 to 72 px
- group label area: 16 to 20 px

The exact pixel values can follow Qt platform metrics, but the conceptual geometry should remain fixed.

The ribbon should not grow vertically because one group contains verbose labels.

---

## Group layout

Each ribbon group should have:

- one subtle outer boundary or separator,
- one group label at the bottom,
- a compact internal grid,
- optional accent indication,
- optional overflow affordance.

Avoid stacking multiple visual boxes.

The current presentation often creates:

1. ribbon background,
2. colored group border,
3. group container,
4. button border,
5. selected-button fill.

This produces too much visual structure.

Prefer one clear group boundary and lightweight controls inside it.

---

# Two-row command grid

The default ribbon group layout should be a two-row grid.

For example:

```text
┌───────────────────────────────┐
│ [icon] [icon] [icon] [icon]  │
│ [icon] [icon] [icon] [icon]  │
│          Bonds                │
└───────────────────────────────┘
```

Standard controls can occupy multiple grid columns:

```text
┌────────────────────────────────────┐
│ [ Move Atom       ] [icon] [icon] │
│ [ Rotate Selection] [icon] [icon] │
│             Selection              │
└────────────────────────────────────┘
```

A large control can span both rows:

```text
┌─────────────────────────────────────┐
│ ┌──────────┐ [ Move Atom          ] │
│ │  SELECT  │ [ Rotate Selection   ] │
│ └──────────┘                        │
│          Select and edit            │
└─────────────────────────────────────┘
```

This pattern is much closer to the useful density of LibreOffice and traditional chemistry editors.

---

# YAML should own presentation

The ribbon renderer should not infer every visual decision from action names or from `primary` versus `supporting`.

Extend `ribbon_layout.yaml` so that the resource can explicitly describe presentation.

A possible entry model:

```yaml
- action: draw.selection.structure
  role: primary
  priority: required
  presentation: large

- action: draw.arrange.move_atom
  role: supporting
  priority: normal
  presentation: standard

- action: draw.bond.solid_wedge
  role: supporting
  priority: normal
  presentation: compact
```

The renderer should support at least:

```yaml
presentation: compact | standard | large
```

Optional additional metadata can support:

```yaml
show_label: true | false
row: 1 | 2
column: integer
span: integer
```

However, explicit row and column coordinates should be added only if automatic packing proves insufficient. Prefer declarative intent over pixel-level layout instructions.

A useful intermediate model is:

```yaml
layout: compact_grid
columns: 4
```

at the group level.

For example:

```yaml
- id: reaction_notation
  label_key: Reaction notation
  accent: reaction
  layout: compact_grid
  columns: 4
  entries:
    - {action: draw.arrow, presentation: compact, priority: required}
    - {action: draw.plus, presentation: compact, priority: required}
    - {action: draw.arrow.equilibrium, presentation: compact}
    - {action: draw.arrow.curved_electron, presentation: compact}
    - {action: draw.arrow.curved_retro, presentation: compact}
    - {action: draw.arrow.curved_reaction, presentation: compact}
    - {action: draw.arrow.curved_equilibrium, presentation: compact}
```

This is a much better representation of reaction notation than two giant buttons plus several long labeled buttons.

---

# Separate command importance from control size

The current schema has useful concepts:

```yaml
role: primary | supporting
priority: required | normal
```

Keep them.

Their meanings should be explicit:

## `role`

Describes importance within the task.

- `primary`: central command for the group
- `supporting`: related command

Role may influence ordering or subtle emphasis.

Role does not dictate physical size.

## `priority`

Describes survival under constrained width.

- `required`: remain directly visible as long as possible
- `normal`: may move to overflow earlier

Priority does not dictate physical size.

## `presentation`

Describes physical rendering.

- `compact`
- `standard`
- `large`

This separation lets Ferrum express cases such as:

```yaml
{action: draw.arrow, role: primary, priority: required, presentation: compact}
```

Draw Arrow is a primary reaction command, but its arrow icon is immediately recognizable and does not need a giant card.

---

# Reduce dependence on "More"

Overflow is a responsive fallback, not a normal group member.

The current Home tab gives several groups a large `More` control. This spends valuable ribbon area on a button whose purpose is to reveal commands that could often fit directly if compact controls were used.

The renderer should first attempt:

1. full group at preferred density,
2. compact alternative presentation,
3. group overflow.

Do not allocate a permanent large `More` card when a small overflow affordance can serve the same function.

A group overflow affordance can be a small chevron in the group header or lower corner.

The group remains visually about chemistry commands rather than about navigation to hidden chemistry commands.

---

# Recommended tab redesign

## Home

Home should contain the highest-frequency cross-task tools.

It should not attempt to provide a large-card version of every major feature.

Suggested organization:

### Select and edit

Large:
- Select Structure

Standard:
- Move Atom
- Rotate Selected Atoms

Compact:
- Scale
- other frequent selection transforms if space permits

### Draw

Large or standard:
- Add Atom
- Draw Bond

Compact:
- solid wedge
- hashed wedge
- connect selected
- next drawing if appropriate

### Rings and templates

Compact:
- common carbon rings
- cyclohexane
- Haworth

Standard:
- Template Catalog

The existing menu hierarchy already defines regular C3 through C8 rings. Those commands are particularly good candidates for a compact icon grid.

### Reaction

Compact:
- arrow
- plus
- equilibrium arrow

Standard:
- Create Reaction
- Reaction Inspector

This would expose more capability while using less width than the current Home tab.

---

# Structure tab

The Structure tab should become the densest chemistry-authoring tab.

## Atoms and bonds

This group should emphasize visual chemistry tools.

A likely arrangement:

```text
[Atom] [Bond] [Wedge] [Hash]
[Connect] [Order] [...]
```

If bond order commands become individually available as actions, prefer direct compact buttons for:

- single
- double
- triple

Likewise, visually distinct bond styles should be directly accessible where practical.

The original BKChem mode configuration explicitly treated bond order and bond type as collections of selectable options. That interaction model is appropriate for a compact ribbon.

## Rings

Expose several rings directly.

For example:

```text
[C3] [C4] [C5] [C6]
[C7] [C8] [Haworth] [...]
```

The ring icon itself should depict the ring. Labels such as "Insert Cyclohexane Ring" should live in tooltips rather than consume permanent ribbon width.

## Groups and templates

Template browsing may legitimately need a standard or larger control because a catalog is a browsing operation rather than a single drawing primitive.

Compact-group placement and attachment can remain smaller.

## Geometry

Geometry operations can use compact or standard controls depending on icon quality.

Do not promote them all to large controls merely because they are commands rather than modes.

---

# Reactions tab

This tab should benefit dramatically from higher density.

The current reaction notation group contains seven actions. Those seven actions should fit comfortably in a compact two-row grid.

Suggested concept:

```text
Reaction notation
┌─────────────────────────────────────┐
│ [→] [+] [⇌] [curved electron]      │
│ [retro] [curved rxn] [curved eq]   │
└─────────────────────────────────────┘
```

Use actual Ferrum icons rather than textual symbols.

The full names remain available through tooltips and the command palette.

### Reaction structure

Use standard controls for:

- Create Reaction
- Reaction Inspector

These are operations rather than drawing primitives, so labels are useful.

### Conditions

Text insertion can be standard or compact depending on how recognizable the icon becomes.

---

# Annotate tab

Annotation should also become substantially denser.

### Labels

Insert Text may remain standard because the command is conceptually broad.

### Brackets and lines

All six current commands can fit in a compact grid:

```text
[rect bracket] [round bracket] [wavy]
[line]         [polyline]      [polygon]
```

### Shapes

All four shapes should be compact:

```text
[rectangle] [square] [oval] [circle]
```

There is little value in permanently displaying "Draw Rectangle", "Draw Square", "Draw Oval", and "Draw Circle" when the icons can literally show those shapes.

---

# View tab

View controls should be compact.

Zoom and grid operations are conventional and do not need large cards.

Suggested:

```text
Zoom
[100%] [+] [-] [Page] [Content]

Grid
[Grid] [Snap]
```

Short labels may be retained where the icon alone is unclear.

---

# Icon design is part of density

Compact controls only work if the icons communicate well.

Ferrum should prioritize chemistry-specific icons that depict the result of the tool.

Examples:

- single bond: one line
- double bond: two lines
- triple bond: three lines
- wedge: filled wedge
- hashed wedge: hashed wedge
- C5 ring: pentagon
- C6 ring: hexagon
- equilibrium arrow: equilibrium arrow
- curved electron arrow: actual curved electron arrow
- rectangle: rectangle
- round bracket: round bracket

Avoid generic icons when the chemistry notation itself can be the icon.

This follows the strongest aspect of BKChem's original interface. Its toolbar was visually dense because many tools were represented directly by their notation.

---

# Labels and tooltips

Compact controls should normally omit permanent labels.

Every compact control must still provide:

- accessible name
- tooltip
- status/help text where Ferrum supports it
- command palette discoverability

For example:

```yaml
action: draw.ring.regular.c6
presentation: compact
show_label: false
```

The QAction can continue to own the canonical label and help text.

The ribbon presentation layer decides whether that label is rendered permanently.

This maintains one command vocabulary without forcing the full vocabulary onto every visible button.

---

# Responsive behavior

Ribbon responsiveness should preserve direct access as long as possible.

Recommended collapse order:

1. retain preferred presentation,
2. convert eligible `standard` controls to compact,
3. remove optional visible labels,
4. reduce spacing between controls within defined limits,
5. move `priority: normal` commands into group overflow,
6. collapse an entire low-priority group only at very narrow widths.

`priority: required` commands should remain visible longer than normal commands.

Do not resize controls continuously. Controls should move between known presentation states so the ribbon remains visually stable.

---

# Visual simplification

The ribbon currently uses colored accent lines effectively to distinguish groups, but the remainder of the chrome should be restrained.

Recommended treatment:

- one neutral ribbon background
- subtle group boundaries
- thin accent at the group top if desired
- minimal button outlines in idle state
- stronger hover and active-tool state
- consistent corner radius
- consistent internal padding
- one group label treatment

Avoid making every command look like an independent card.

Drawing tools should visually read as a toolbar inside an organized ribbon group.

---

# Active tool state

Chemistry drawing applications depend heavily on persistent modes.

The active drawing tool must therefore be more visually obvious than an ordinary command that was merely clicked.

Use a consistent selected-tool state across compact, standard, and large controls.

For example:

- filled or tinted background
- clear border
- retained state until another mode is chosen

Do not use physical button size to indicate active state.

The distinction is semantic and should be represented by state styling.

---

# Relationship to menus

The menu and ribbon should remain separate presentations of the same canonical actions.

`menus.yaml` is already broad and hierarchical. For example, Draw contains atoms and bonds, rings and structures, reaction notation, annotation, shapes, coordinate generation, geometry repair, arrangement, and active-tool commands.

The ribbon does not need to reproduce that hierarchy exactly.

Instead:

- menus provide comprehensive command access,
- ribbon provides task-oriented high-frequency access,
- command palette provides searchable access,
- contextual UI provides selection-specific access.

All of them should reference the same action IDs.

This is the architectural advantage Ferrum should preserve.

---

# Relationship to legacy BKChem modes

The legacy `modes.yaml` should be treated as useful interaction evidence, not as the new architecture.

It shows that BKChem grouped many dense tool families:

- bond order
- bond type
- atom marks
- arrow types
- bracket styles
- transformation modes
- vector shapes
- repair operations

Those families should inform Ferrum's ribbon organization.

Do not recreate the legacy mode system merely to reproduce its toolbar.

Instead, expose equivalent Ferrum actions through compact declarative ribbon groups.

---

# Suggested YAML evolution

A reasonable next schema could look like:

```yaml
tabs:
  - id: structure
    label_key: Structure
    groups:
      - id: atoms_bonds
        label_key: Atoms and bonds
        accent: drawing
        layout: compact_grid
        rows: 2
        entries:
          - action: draw.atom_at_point
            role: primary
            priority: required
            presentation: large

          - action: draw.bond
            role: primary
            priority: required
            presentation: large

          - action: draw.bond.solid_wedge
            role: supporting
            priority: required
            presentation: compact

          - action: draw.bond.hashed_wedge
            role: supporting
            priority: required
            presentation: compact

          - action: draw.bond.connect_selected
            role: supporting
            priority: normal
            presentation: compact

          - action: edit.bond.change_order
            role: supporting
            priority: normal
            presentation: compact

      - id: rings
        label_key: Rings
        accent: structure
        layout: compact_grid
        rows: 2
        entries:
          - {action: draw.ring.regular.c3, presentation: compact}
          - {action: draw.ring.regular.c4, presentation: compact}
          - {action: draw.ring.regular.c5, presentation: compact}
          - {action: draw.ring.regular.c6, presentation: compact, priority: required}
          - {action: draw.ring.regular.c7, presentation: compact}
          - {action: draw.ring.regular.c8, presentation: compact}
          - {action: draw.ring.haworth.insert, presentation: compact}
```

This example is intentionally about presentation. It does not change action ownership or command behavior.

---

# Renderer requirements

The ribbon renderer should:

1. Read presentation metadata from `ribbon_layout.yaml`.
2. Resolve every action through the existing ActionRegistry.
3. Render compact, standard, and large controls from the same QAction.
4. Pack compact controls into two rows.
5. Keep group height consistent across the tab.
6. Keep group labels aligned.
7. Provide tooltips and accessible names for icon-only controls.
8. Represent persistent tool state consistently.
9. Use overflow only when width actually requires it.
10. Preserve command identity when the same QAction appears in menus, ribbon, contextual menus, or the command palette.

The renderer should not acquire command behavior, chemistry logic, document mutation, or a second action vocabulary.

---

# Acceptance criteria

The redesign is successful when:

- A normal desktop-width Ferrum window exposes substantially more direct commands than the current ribbon.
- The ribbon command area uses approximately two compact rows rather than predominantly large cards.
- Visually recognizable chemistry primitives are normally icon-first.
- Rings, reaction arrows, shapes, brackets, bond styles, and similar tool families can be scanned visually.
- `primary` no longer implies a large button.
- `priority` controls overflow behavior rather than button dimensions.
- Only a small number of commands on each tab use large presentation.
- Group heights and labels remain visually aligned.
- Large permanent `More` cards are no longer a normal part of the layout.
- The ribbon remains driven by YAML and ActionRegistry action IDs.
- Menus and ribbon continue to present the same canonical commands without duplicating command behavior.
- Tooltips and accessible names make compact icon-only controls understandable and accessible.
- Active drawing modes remain visually obvious.
- Narrow-window behavior degrades through discrete presentation states rather than arbitrary shrinking.

---

# Initial implementation priority

Implement density in this order:

1. Add `presentation: compact | standard | large` to ribbon entries.
2. Decouple `role` from physical button size.
3. Add a two-row compact group layout.
4. Convert obvious chemistry primitives to compact presentation.
5. Replace permanent large `More` controls with lightweight group overflow.
6. Reduce group chrome and button borders.
7. Tune responsive collapse using `priority`.
8. Review icon quality for commands that now rely primarily on pictograms.

Start with the **Reactions** and **Annotate** tabs because they provide clear test cases. Reaction arrows, brackets, lines, and shapes should demonstrate immediately whether the compact presentation model works.

Then apply the same system to **Structure**, where rings, bond types, and geometry tools provide the largest density benefit.

Finally, tune **Home** after the compact system is proven. Home should be a curated high-frequency surface rather than the place where the layout system is invented.

---

# Design constraint

The goal is not to reproduce either BKChem or LibreOffice visually.

The goal is to preserve what each interface does well:

- BKChem demonstrates that chemistry tools can be extremely information dense.
- A ribbon provides task grouping and discoverability.
- Ferrum's YAML and ActionRegistry architecture provides one canonical command system with multiple presentations.

Ferrum should combine those strengths into a compact chemistry-specific ribbon.
