# Wave Function Collapse (WFC)

Wave Function Collapse overlapping model implementation in Rust. Generates novel images by extracting NxN patterns from samples and synthesizing outputs that follow the same adjacency constraints.

<p align="center">
<<<<<<< HEAD
  <img src="samples/rooms.png" width="45%" style="image-rendering: pixelated;" />
  <img src="examples/wfc-rooms-output.png" width="45%" />
=======
  <img src="samples/rooms-scaled.png" width="35%" />
  <img src="examples/wfc-rooms-output.png" width="35%" />
>>>>>>> c16d3e8 (docs: add README and MIT license)
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
<<<<<<< HEAD
  <img src="examples/wfc-rooms-animation.gif" width="45%" />
  <img src="examples/wfc-maze-animation.gif" width="45%" />
=======
  <img src="examples/wfc-rooms-animation.gif" width="40%" />
  <img src="examples/wfc-maze-animation.gif" width="40%" />
>>>>>>> c16d3e8 (docs: add README and MIT license)
</p>

### Edge Constraints

Let's see how edge constraints affect the output.

**Sample:**

<p align="center">
<<<<<<< HEAD
  <img src="samples/more-flowers.png" width="30%" style="image-rendering: pixelated;" />
</p>

**Results:**

<p align="center">
  <img src="examples/wfc-more-flowers-raw-output.png" width="30%" />
  <img src="examples/wfc-more-flowers-vertical-output.png" width="30%" />
  <img src="examples/wfc-more-flowers-vertical-sides-output.png" width="30%" />
</p>

**Left to right:** Without constraints, with vertical constraints, with vertical and sides constraints
=======
  <img src="samples/more-flowers-scaled.png" width="30%" />
</p>

|              Without constraints              |                Vertical constraints                |               Vertical + Sides constraints               |
| :-------------------------------------------: | :------------------------------------------------: | :------------------------------------------------------: |
| ![](examples/wfc-more-flowers-raw-output.png) | ![](examples/wfc-more-flowers-vertical-output.png) | ![](examples/wfc-more-flowers-vertical-sides-output.png) |
>>>>>>> c16d3e8 (docs: add README and MIT license)

## License

MIT
