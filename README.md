# Wave Function Collapse (WFC)

Wave Function Collapse overlapping model implementation in Rust. Generates novel images by extracting NxN patterns from samples and synthesizing outputs that follow the same adjacency constraints.

<p align="center">
  <img src="samples/rooms-scaled.png" width="35%" />
  <img src="examples/wfc-rooms-output.png" width="35%" />
</p>

## Algorithm

The overlapping model extracts all NxN pattern tiles from a sample
image and records which patterns can appear adjacent to each other.
Starting from a completely undetermined output, it iteratively:

1. Finds the cell with lowest entropy (fewest possible patterns)
2. Collapses it to a random valid pattern
3. Propagates constraints to neighboring cells
4. Repeats until complete or a contradiction occurs

## GUI

```bash
cargo run --release --bin wfc-egui
```

![WFC GUI Screenshot](examples/egui-screenshot.png)

## Results

<p align="center">
  <img src="examples/wfc-rooms-animation.gif" width="40%" />
  <img src="examples/wfc-maze-animation.gif" width="40%" />
</p>

### Edge Constraints

Let's see how edge constraints affect the output.

**Sample:**

<p align="center">
  <img src="samples/more-flowers-scaled.png" width="30%" />
</p>

|              Without constraints              |                Vertical constraints                |               Vertical + Sides constraints               |
| :-------------------------------------------: | :------------------------------------------------: | :------------------------------------------------------: |
| ![](examples/wfc-more-flowers-raw-output.png) | ![](examples/wfc-more-flowers-vertical-output.png) | ![](examples/wfc-more-flowers-vertical-sides-output.png) |

## License

MIT
