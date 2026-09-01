# YAML resource format

Ferrum packages three YAML resource families that define the visible desktop command surfaces and
appearance. They are maintainership inputs shipped with `ferrum-qt`, not molecular interchange,
saved-document data, user preferences, or a public plugin format. Rust remains the authority for
chemistry, document state, and rendering facts; the action registry remains the authority for
command behavior, checked state, enabled state, shortcuts, and Qt object lifetime.

The authoritative files are:

- [`menus.yaml`](../packages/ferrum-chem-qt.app/ferrum_qt/resources/menus.yaml) for ordinary and
  context-menu placement;
- [`ribbon_layout.yaml`](../packages/ferrum-chem-qt.app/ferrum_qt/resources/ribbon_layout.yaml)
  for task-ribbon placement; and
- [`themes/light.yaml`](../packages/ferrum-chem-qt.app/ferrum_qt/resources/themes/light.yaml) and
  [`themes/dark.yaml`](../packages/ferrum-chem-qt.app/ferrum_qt/resources/themes/dark.yaml) for
  shipped appearance palettes.

Use [FILE_FORMATS.md](FILE_FORMATS.md) for CDML and molecular-interchange formats. Do not use
these YAML resources to add a chemistry operation, create a second `QAction`, define a shortcut,
or persist a document fact. Add the action at its feature owner first; then place its existing
registered client here.

## Loading and validation

The neutral packaged-resource loader reads YAML with `yaml.safe_load` and returns only its parsed
data; resource-specific validators own the schemas. Window startup performs a failure-atomic
preflight before it constructs either a menu or ribbon client: each static action must be an
existing registered action, and each dynamic menu must be an existing registered dynamic-menu
client. Ribbon loading then resolves every declared action to that same existing `QAction`.
Invalid menu or ribbon resources raise `DeclarativeResourceError`; they are never silently
repaired or partially applied.

The menu and ribbon schemas reject unknown keys, empty required strings and lists, duplicate
declaration IDs, duplicate action placements within their respective surface, and unresolved
action or dynamic-menu IDs. Their declared order is visible order. A command palette derives its
breadcrumb from the ordinary menu when available and otherwise from the ribbon.

Theme names are filenames without `.yaml` (`light`, `dark`). The complete `document_display`
mapping is parsed into an immutable `DocumentDisplayPaletteV1`; invalid, transparent, incomplete,
or insufficient-contrast display colors are refused before the palette is issued to the document
surface.

## Menu resource

`menus.yaml` is exactly one mapping with nonempty `menus` and `contexts` lists:

```yaml
menus:
  - id: draw
    label_key: Draw
    help_key: Canvas authoring commands
    items:
      - section:
          id: draw_shapes
          label_key: Shapes
          items:
            - action: draw.vector.rectangle
            - separator: true
            - submenu:
                id: draw_regular_ring
                label_key: Insert Regular Ring...
                help_key: Insert a regular carbon ring
                items:
                  - action: draw.ring.regular.c6
      - dynamic_menu: file.recent
contexts:
  - id: selected_structure
    accessible_name: Selected structure actions
    groups:
      - id: inspect
        actions:
          - chemistry.report.molecule
```

Each ordinary top-level menu requires exactly `id`, `label_key`, `help_key`, and `items`.
Its ordered `items` may contain exactly one of these forms:

| Form | Required value | Meaning |
| --- | --- | --- |
| `action` | nonempty registered action ID | Place one static feature-owned QAction. |
| `dynamic_menu` | nonempty registered dynamic-menu ID | Place one changing feature-owned QMenu. |
| `separator` | literal `true` | Place a visual separator. |
| `section` | mapping with `id`, `items`, and optional `label_key` | Group ordered entries without a nested QMenu. |
| `submenu` | mapping with `id`, `label_key`, `help_key`, and `items` | Place a labelled nested QMenu. |

Every section or submenu recursively uses the same item forms. Menu-node IDs are unique across the
whole ordinary menu tree. An action ID and dynamic-menu ID each have at most one ordinary-menu
placement. `label_key`, `help_key`, and all IDs are nonempty strings; they are currently visible
English text or stable dotted action identifiers, not a localization-catalog indirection.

Each context declaration requires exactly `id`, `accessible_name`, and `groups`. A group requires
exactly `id` and a nonempty ordered `actions` list. Context action IDs must exist in the action
registry and appear at most once in that context. Context placement supplements the ordinary menu;
it does not create a separate command implementation.

## Ribbon resource

`ribbon_layout.yaml` is exactly a mapping with nonempty `quick_access`,
`global_actions`, and `tabs` lists:

```yaml
quick_access:
  - file.new
  - file.open
  - file.save
  - edit.undo
  - edit.redo
global_actions:
  - view.command_palette
tabs:
  - id: structure
    label_key: Structure
    groups:
      - id: atoms_bonds
        label_key: Atoms and bonds
        overflow_label_key: More atom and bond commands
        accent: drawing
        entries:
          - action: draw.atom_at_point
            role: primary
            priority: required
          - action: draw.bond
            role: supporting
            priority: normal
```

The two header lists contain ordered registered action IDs. Their IDs are
unique across both lists. Quick access is the persistent icon-only route for
universal file/history commands; global actions are persistent labelled
discovery routes.

A tab requires exactly `id`, `label_key`, and `groups`. A group requires exactly `id`,
`label_key`, `overflow_label_key`, `accent`, and `entries`. An entry requires exactly `action`,
`role`, and `priority`.

- Tab IDs are unique across the resource; group IDs are unique within a tab; action IDs are unique
  within a tab.
- `role` is either `primary` or `supporting`.
- `priority` is either `required` or `normal`.
- `accent` is one of `annotation`, `drawing`, `reaction`, `selection`, `structure`, `utility`, or
  `view`. It is semantic grouping metadata, not a literal color.
- Every action must resolve to an already bound QAction. The ribbon only projects that client; it
  cannot change its behavior or state.
- Entry order controls the visual and keyboard traversal order. The responsive ribbon reduces
  later declared groups first when space is constrained, so retain deliberate task order.

## Theme resource

Each shipped theme is a top-level mapping with a visible `name` and display layers:

```yaml
name: Light
document_display:
  canvas_surround: "#596675"
  page_fill: "#ffffff"
  page_outline: "#71717a"
  document_foreground: "#1f2328"
  atom_number: "#0056b3"
  selection_outline: "#0066cc"
  hover_outline: "#0077b6"
  preview_outline: "#5e3bb0"
  preview_fill: "#4a8ac7"
  keyboard_cursor: "#7024a8"
  grid_line: "#87939d"
  grid_dot_outline: "#77848e"
  grid_dot_fill: "#5b8f7c"
  elements: {}
ribbon: {}
gui: {}
canvas: {}
chemistry: {}
paper: {}
grid: {}
```

`document_display` is the closed current contract. It must contain exactly the thirteen named role
colors above plus `elements`; `elements` must be `{}` in V1. Every role color must be an opaque,
valid Qt color. `canvas_surround` and `page_fill` have no contrast floor. Against `page_fill`, the
thin roles `document_foreground`, `atom_number`, `selection_outline`, `hover_outline`,
`preview_outline`, and `keyboard_cursor` require 4.5:1; `page_outline`, `preview_fill`,
`grid_line`, `grid_dot_outline`, and `grid_dot_fill` require 3.0:1. Do not invent element colors:
the V1 display palette intentionally has no element-role map.

The closed `ribbon` mapping contains exactly `shell`, `header_bg`, `header_fg`, `header_muted`,
`tab_hover`, `tab_active_bg`, `tab_active_fg`, `surface`, `group_bg`, `group_border`,
`button_hover`, `button_checked`, `button_checked_border`, `caption_fg`, `context_bg`,
`context_fg`, `focus`, and one `accent_*` color for each closed accent value above. Every value is
a valid Qt color. Ribbon text pairs require 4.5:1 contrast; focus, checked-state borders, and
accent rails require 3:1 against their painted surfaces.

The `gui`, `canvas`, `chemistry`, `paper`, and `grid` mappings are shipped Qt appearance inputs for
the remaining application chrome and presentation adapters. Keep their existing keys consistent
across the light and dark themes. The strict document-display contract, rather than these adapter
mappings, is the source of colors for the Rust-backed document surface.

## Safe maintenance workflow

1. Add or change the feature-owned action and its focused tests before adding its YAML placement.
2. Change only the authoritative resource file; do not duplicate menu, ribbon, or palette facts in
   Python widgets.
3. Keep stable dotted action IDs and resource IDs. Ferrum is pre-production, so rename an obsolete
   identifier consistently instead of preserving an alias.
4. Run the resource and palette tests, then the repository documentation checks:

   ```bash
   source source_me.sh && python3 -m pytest \
     packages/ferrum-chem-qt.app/tests/test_declarative_resources.py \
     packages/ferrum-chem-qt.app/tests/test_document_display_palette.py
   source source_me.sh && python3 -m pytest \
     tests/test_markdown_links.py \
     tests/test_ascii_compliance.py \
     tests/test_source_file_line_limit.py
   ```

For current user-visible command organization, see [GUI_TOUR.md](GUI_TOUR.md). For ownership and
the Rust-to-Qt boundary, see [CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md).
