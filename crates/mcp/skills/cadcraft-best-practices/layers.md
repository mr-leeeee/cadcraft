# Layer Management Best Practices

## Standard Layer Naming Convention

| Prefix | Purpose | Color | Example |
|--------|---------|-------|---------|
| `A-` | Architecture | 1-7 | `A-WALL`, `A-DOOR`, `A-WINDOW` |
| `S-` | Structure | 1-7 | `S-COLUMN`, `S-BEAM` |
| `M-` | Mechanical | 1-7 | `M-DUCT`, `M-PIPE` |
| `E-` | Electrical | 1-7 | `E-LIGHT`, `E-POWER` |
| `P-` | Plumbing | 1-7 | `P-WATER`, `P-GAS` |
| `X-` | Non-plotting (defpoints, etc.) | 8 | `X-REF`, `X-VPORT` |
| `0` | Default/Construction | 7 | (built-in) |

## Layer Properties

### Recommended Default Properties
```json
{
  "A-WALL": { "color": "red", "linetype": "continuous", "lineweight": "0.35", "plot": true },
  "A-DOOR": { "color": "green", "linetype": "continuous", "lineweight": "0.25", "plot": true },
  "A-WINDOW": { "color": "cyan", "linetype": "continuous", "lineweight": "0.25", "plot": true },
  "A-TEXT": { "color": "white", "linetype": "continuous", "lineweight": "0.18", "plot": true },
  "A-DIM": { "color": "yellow", "linetype": "continuous", "lineweight": "0.18", "plot": true },
  "A-HATCH": { "color": "8", "linetype": "continuous", "lineweight": "0.05", "plot": true },
  "X-REF": { "color": "8", "linetype": "continuous", "lineweight": "0.18", "plot": false },
  "X-VPORT": { "color": "8", "linetype": "continuous", "lineweight": "0.00", "plot": false },
  "DEFPOINTS": { "color": "8", "linetype": "continuous", "lineweight": "0.00", "plot": false }
}
```

## Layer State Management

| State | When to Use |
|-------|-------------|
| **On/Off** | Toggle visibility quickly; off = invisible but selectable in some contexts |
| **Freeze/Thaw** | Better for performance; frozen = not processed, not selectable |
| **Lock/Unlock** | Prevent accidental edits; locked = visible but not editable |
| **Plot/No Plot** | Control output; no-plot for reference layers |

### Recommended Workflow
1. **Create all layers** at drawing start with `layer.new`
2. **Set properties** (color, linetype, lineweight) immediately
3. **Set current layer** before drawing: `layer.set {name: "A-WALL"}`
4. **Freeze reference layers** (xrefs, images) for performance
4. **Lock dimension/text layers** when doing heavy geometry edits
5. **Use layer states** (save/restore) for different view configurations

## Layer Filters
Create layer filters for quick isolation:
- `All_Architecture` → `A-*`
- `All_Structure` → `S-*`
- `Plot_Layers` → exclude `X-*`, `DEFPOINTS`
- `Edit_Layers` → exclude locked/frozen

## Agent Guidelines
- Always query `layer.list` before creating new layers
- Use semantic layer names, not generic "Layer1", "Layer2"
- Set lineweight by layer, not by entity (except overrides)
- Keep layer 0 for blocks only (entities in blocks inherit layer 0 properties)
- Don't draw on layer 0 directly