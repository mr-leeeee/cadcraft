# CADCraft 2D Drafting Reference

## Quick Command Reference

### Drawing Entities
| Command | Alias | Description | Key Parameters |
|---------|-------|-------------|----------------|
| `line` | `l` | Draw line segments | `points: [[x,y],...]` |
| `circle` | `c` | Draw circle | `center: [x,y], radius: n` or `diameter: n` |
| `arc` | `a` | Draw arc | `p1,p2,p3` or `center,radius,start,end` |
| `rectang` | `rec` | Draw rectangle | `p1,p2` or `dimensions: [w,h]` |
| `pline` | `pl` | Polyline | `vertices: [[x,y],...]` |
| `polygon` | `pol` | Regular polygon | `sides, center, radius` |
| `ellipse` | `el` | Ellipse | `center, major: [dx,dy], ratio` |
| `point` | `po` | Point | `at: [x,y]` |
| `spline` | `spl` | Spline | `fit: [[x,y],...]` |
| `donut` | `do` | Donut (wide polyline) | `center, inside, outside` |
| `text` | `dt` | Single-line text | `at, text, height, rotation` |
| `mtext` | `t,mt` | Multi-line text | `at, text, width, height` |

### Modify Commands
| Command | Alias | Description |
|---------|-------|-------------|
| `move` | `m` | Move objects |
| `copy` | `co,cp` | Copy objects |
| `rotate` | `ro` | Rotate objects |
| `scale` | `sc` | Scale objects |
| `mirror` | `mi` | Mirror objects |
| `trim` | `tr` | Trim objects |
| `extend` | `ex` | Extend objects |
| `offset` | `o` | Offset objects |
| `array` | `ar` | Rectangular/polar array |
| `erase` | `e` | Delete objects |
| `explode` | `x` | Explode blocks/polylines |
| `fillet` | `f` | Fillet corners |
| `chamfer` | `cha` | Chamfer corners |
| `stretch` | `s` | Stretch objects |
| `lengthen` | `len` | Change length |
| `break` | `br` | Break object |
| `join` | `j` | Join objects |

### Layers & Properties
| Command | Description |
|---------|-------------|
| `layer.new` | Create layer |
| `layer.set` | Set current layer |
| `layer.off` | Turn layer off |
| `layer.freeze` | Freeze layer |
| `layer.lock` | Lock layer |
| `layer.color` | Set layer color |
| `layer.list` | List layers |
| `properties` | Modify properties |
| `matchprop` | Match properties |

### Dimensions & Annotation
| Command | Description |
|---------|-------------|
| `dimlinear` | Linear dimension |
| `dimaligned` | Aligned dimension |
| `dimradius` | Radius dimension |
| `dimdiameter` | Diameter dimension |
| `dimangular` | Angular dimension |
| `dimbaseline` | Baseline dimensions |
| `dimcontinue` | Continue dimensions |
| `dimedit` | Edit dimensions |
| `qdim` | Quick dimensions |
| `leader` | Leader with text |
| `mleader` | Multileader |

### Blocks & References
| Command | Description |
|---------|-------------|
| `block` | Create block |
| `insert` | Insert block |
| `wblock` | Write block to file |
| `xref` | External reference |
| `xbind` | Bind xref |
| `explode` | Explode block |

### View & Navigation
| Command | Alias | Description |
|---------|-------|-------------|
| `zoom` | `z` | Zoom (extents, window, center, scale, pan) |
| `pan` | `p` | Pan view |
| `view` | `v` | Named views |
| `vports` | | Viewports |

### Selection
| Method | Description |
|--------|-------------|
| Click | Single select |
| Window (left→right) | Select inside |
| Crossing (right→left) | Select crossing |
| Fence | Select along line |
| Lasso | Freeform selection |
| `select.all` | Select all |
| `select.window` | Window selection |
| `select.crossing` | Crossing selection |

## Coordinate Input Formats

| Format | Example | Description |
|--------|---------|-------------|
| Absolute | `10,20` | World coordinates |
| Relative | `@5,10` | From last point |
| Polar | `@25<45` | Distance & angle |
| Direct Distance | `25` | At current angle |

## Common Workflows

### Create a Drawing Template
```
new {metric: true}
layer.new {name: "Walls", color: "red", current: true}
layer.new {name: "Doors", color: "green"}
layer.new {name: "Text", color: "white"}
```

### Draw a Room
```
rectang {p1: [0,0], p2: [5000,4000]}
offset {distance: 200, inside: true}  // inner wall
line {points: [[1000,0], [1000,2000]]}  // door opening
```

### Add Dimensions
```
dimlinear {p1: [0,0], p2: [5000,0], dimline: [0,-500]}
dimaligned {p1: [0,0], p2: [0,4000], dimline: [-500,0]}
```

### Array Objects
```
circle {center: [1000,1000], radius: 100}
array {count: 5, spacing: 500, angle: 0}  // rectangular
// or
array {center: [2500,2000], count: 8, angle: 360}  // polar
```

## Entity Info Commands (for agents)
| Command | Returns |
|---------|---------|
| `drawing.inspect` | Full drawing summary |
| `entities` | Filtered entity list |
| `entity.info {handle}` | Single entity details |
| `layer.list` | All layers |
| `block.list` | All blocks |
| `view.list` | Named views |
| `selection.get` | Current selection |
| `undo.info` | Undo stack depth |

## Tips for AI Agents

1. **Use `batch` for atomic operations** - Multiple commands with auto-rollback
2. **Query before modify** - Use `entities` or `drawing.inspect` to find handles
3. **Prefer `execute` over `command_line`** - Structured JSON, no prompts
4. **Use `measure` for verification** - Check distances, areas after drawing
4. **Work in metric** - `new {metric: true}` avoids unit confusion
5. **Layer management** - Create layers first, set current before drawing
6. **Zoom extents** - `zoom {mode: "extents"}` after drawing to verify