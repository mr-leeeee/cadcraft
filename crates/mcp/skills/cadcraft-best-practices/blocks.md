# Block Best Practices

## Block Creation

### When to Use Blocks
- Repeated symbols (doors, windows, fixtures, equipment)
- Standard details (title blocks, north arrows, scale bars)
- Annotations that appear multiple times
- Any geometry used 3+ times in a drawing

### Block Naming
| Pattern | Example |
|---------|---------|
| `BLK_<category>_<name>` | `BLK_DOOR_SINGLE`, `BLK_WINDOW_CASMENT` |
| `<discipline>_<type>_<variant>` | `A_DOOR_SINGLE_900`, `M_DIFFUSER_SQUARE` |
| Keep names short, uppercase, underscore-separated |

### Block Definition Process
```
1. Draw geometry on layer 0 (so it inherits layer properties)
2. Create block: block {name: "BLK_DOOR_SINGLE", base: [0,0], entities: [...]}
3. Insert instances: insert {name: "BLK_DOOR_SINGLE", point: [x,y], rotation: 0, scale: 1}
```

### Base Point Placement
- **Insertion point** at logical location (e.g., door hinge, window corner, equipment center)
- **0,0** for symbols centered at origin
- Consistent base points for same-category blocks

## Block Attributes

### Attribute Definitions
```json
{
  "tag": "ROOM_NAME",
  "prompt": "Room Name: ",
  "value": "OFFICE",
  "position": [0, -100],
  "height": 250,
  "layer": "A-TEXT"
}
```

### Best Practices
- Define attributes on layer 0 or A-TEXT
- Use meaningful tags (ROOM_NUM, DOOR_TYPE, EQUIP_TAG)
- Set default values for common cases
- Make attributes invisible if not always needed
- Use `attdef` command for attribute definitions

## Dynamic Blocks (Parametric)

### Parameters to Consider
- **Visibility states** - multiple representations in one block
- **Linear parameters** - stretch length/width
- **Rotation parameters** - flip/rotate
- **Array parameters** - repeat elements

### When to Use
- Door swings (left/right, single/double)
- Window types (casement, sliding, fixed)
- Furniture variations
- Equipment with size options

## Block Management

### Block List Query
```json
block.list {}  // Returns all blocks with entity counts
```

### Purging Unused Blocks
```json
purge {blocks: true}  // Remove unused block definitions
```

### Block Editing
- Use `bedit` for in-place block editing
- Changes propagate to all instances
- Use `block.replace` to swap block definitions

## External References (Xrefs)

### Xref vs Block
| | Xref | Block |
|---|------|-------|
| **Source** | External file | Internal definition |
| **Updates** | Auto-reload | Manual redefine |
| **Collaboration** | Multiple files | Single file |
| **Nesting** | Can nest | Can nest |

### Xref Best Practices
- Attach (not overlay) for design references
- Use relative paths
- Set insertion point to 0,0,0
- Don't explode xrefs (breaks link)
- Bind when finalizing for delivery

## Agent Guidelines
1. Check `block.list` before creating new blocks
2. Use blocks for anything repeated 3+ times
3. Define on layer 0 for layer inheritance
4. Set logical base points
5. Add attributes for data extraction
6. Name consistently with discipline prefix
7. Purge unused blocks before saving