# Dimension Best Practices

## Dimension Styles

### Standard Dimension Style (ISO/DIN)
```json
{
  "name": "ISO-25",
  "dimscale": 1.0,
  "dimtxt": 2.5,
  "dimasz": 2.5,
  "dimexe": 1.25,
  "dimexo": 0.625,
  "dimgap": 0.625,
  "dimtol": 0,
  "dimdec": 0,
  "dimround": 0,
  "dimzin": 0,
  "dimalt": 0,
  "dimaltd": 2,
  "dimtofl": 1,
  "dimtih": 0,
  "dimtoh": 0,
  "dimse1": 0,
  "dimse2": 0,
  "dimtad": 0,
  "dimjust": 0,
  "dimtmove": 0,
  "dimtix": 0,
  "dimsoxd": 0,
  "dimclrd": 7,
  "dimclre": 7,
  "dimclrt": 7,
  "dimtfac": 1.0,
  "dimgap": 0.625
}
```

### Architectural Dimension Style
```json
{
  "name": "ARCH-1/4",
  "dimscale": 48.0,  // 1/4" = 1'-0"
  "dimtxt": 0.125,
  "dimasz": 0.125,
  "dimexe": 0.0625,
  "dimexo": 0.03125,
  "dimgap": 0.03125,
  "dimdec": 0,
  "dimfrac": 1,  // fractional inches
  "dimzin": 8,   // suppress zero feet/inches
  "dimunit": 2,  // architectural
  "dimpost": "\"",
  "dimsuffix": ""
}
```

## Dimension Types & When to Use

| Type | Command | Use Case |
|------|---------|----------|
| Linear | `dimlinear` | Horizontal/vertical distances |
| Aligned | `dimaligned` | True distance between two points |
| Radius | `dimradius` | Circle/arc radius |
| Diameter | `dimdiameter` | Circle/arc diameter |
| Angular | `dimangular` | Angle between lines |
| Baseline | `dimbaseline` | Multiple dims from same baseline |
| Continue | `dimcontinue` | Chain of dimensions |
| Ordinate | `dimordinate` | X/Y from origin |
| Arc Length | `dimarc` | Arc length |

## Placement Rules

### Spacing
- **First dimension line**: ≥ 10mm (3/8") from object
- **Subsequent lines**: ≥ 6mm (1/4") apart
- **Parallel dimensions**: Stagger to avoid overlap

### Orientation
- **Text parallel** to dimension line (default)
- **Text horizontal** for readability (set `dimtih = 1`)
- **Above dimension line** with gap (`dimgap`)

### Text Position
- **Centered** between extension lines (default)
- **Above** dimension line for horizontal dims
- **Left/Right** aligned for vertical dims

## Dimensioning Rules

### What to Dimension
1. **Functional dimensions** - sizes that affect function/fit
2. **Overall dimensions** - total length, width, height
3. **Location dimensions** - positions of features
4. **Repetitive features** - dimension once, note "TYP" or "×N"

### What NOT to Dimension
- Redundant dimensions (derivable from others)
- Manufacturing-only dimensions (unless specified)
- Dimensions that clutter the drawing

### Tolerances
```json
{
  "dimtol": 1,
  "dimtp": 0.05,   // upper tolerance
  "dimtm": -0.05,  // lower tolerance
  "dimdec": 3      // decimal places
}
```

## Dimension Editing

### Common Edits
| Action | Command |
|--------|---------|
| Move text | `dimedit {action: "move", handle: "xxx", point: [x,y]}` |
| Rotate text | `dimedit {action: "rotate", handle: "xxx", angle: 45}` |
| Edit text | `dimedit {action: "text", handle: "xxx", text: "NEW"}` |
| Add prefix/suffix | `dimedit {action: "prefix/suffix", handle: "xxx", text: "TYP"}` |
| Flip arrow | `dimedit {action: "flip", handle: "xxx"}` |

### Override Properties (per dimension)
```json
{
  "dimclrd": 1,    // dimension line color
  "dimclre": 1,    // extension line color
  "dimclrt": 1,    // text color
  "dimltype": "dashed",  // line type override
  "dimlwd": "0.50"       // line weight override
}
```

## Dimension Layers

### Recommended Layer Structure
| Layer | Contents | Color | Plot |
|-------|----------|-------|------|
| `A-DIM` | Dimension lines, arrows, text | Yellow (2) | Yes |
| `A-DIM-EXT` | Extension lines | 8 (gray) | Yes |
| `A-DIM-HIDDEN` | Hidden dims (reference) | 8 (gray) | No |

### Layer Control
```json
// Set dimension layer
dimstyle.set {name: "ISO-25", dimlayer: "A-DIM", dimExtensionLayer: "A-DIM-EXT"}
```

## Dimension Checking (Agent Verification)

### Use `measure` Tool
```json
{
  "items": [
    {"type": "entity", "handle": "dim1"},
    {"type": "entity", "handle": "dim2"}
  ]
}
// Returns actual distances for comparison
```

### Verification Checklist
- [ ] Dimension values match actual geometry
- [ ] No duplicate dimensions
- [ ] All critical features dimensioned
- [ ] Tolerances specified where needed
- [ ] Text readable (not overlapping)
- [ ] Extension lines don't cross object lines
- [ ] Dimension lines don't cross each other
- [ ] Correct dimension style applied
- [ ] Units consistent across drawing

## Leader & Multileader Best Practices

### Leader
- Use for notes pointing to features
- Arrow at object, text at end
- Keep leader line straight or single bend

### Multileader (Preferred)
```json
mleader {point: [x,y], text: "NOTE: ...", leader: {points: [[x1,y1], [x2,y2]]}}
```
- Block-based content (can include blocks)
- Multiple leader lines to same text
- Better text formatting
- Easier to edit/move

## Agent Guidelines
1. Create dimension style first, set as current
2. Use `dimlinear`/`dimaligned` for most dimensions
3. Place dimensions systematically (outside to inside)
4. Use baseline/continue for repetitive features
5. Verify with `measure` after placing
6. Keep dimensions on dedicated layers
7. Don't explode dimensions (loses associativity)
8. Use multileaders for notes
9. Set `dimscale` to match viewport scale