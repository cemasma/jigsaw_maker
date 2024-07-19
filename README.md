# jigsaw_maker

jigsaw maker project creates jigsaw pieces by given image.

# Usage
1. Build the application
```sh
cargo build --release
```
You can now find the executable of the application inside the ```target/release``` folder. Thereby, you can set its directory to your ```PATH``` or you can run it locally.

2. Run the application.
```sh
jigsaw_maker --file sample.jpg
```
jigsaw_maker will create 4x4 jigsaw pieces by given picture and save those pieces into ```out``` directory.

# Commands
| Command | Value | Description |
|--|--|--|
| -f, --file | String | File path option, required. |
| --column | Integer | Column count option, default value is 4 |
| --row | Integer | Row count option, default value is 4 |
| -h, --help |  | Print help|

# Example
![Sample image](sample.jpg)


|   |   |   |   |
| ------------ | ------------ | ------------ | ------------ |
| ![0 0](out/0:0.png)  | ![1 0](out/1:0.png)  | ![2 0](out/2:0.png)  | ![3 0](out/3:0.png)  |
| ![0 1](out/0:1.png)  | ![1 1](out/1:1.png)  | ![2 1](out/2:1.png)  | ![3 1](out/3:1.png)  |
| ![0 2](out/0:2.png)  | ![1 2](out/1:2.png)  | ![2 2](out/2:2.png)  | ![3 2](out/3:2.png)  |
| ![0 3](out/0:3.png)  | ![1 3](out/1:3.png)  | ![2 3](out/2:3.png)  | ![3 3](out/3:3.png)  |
