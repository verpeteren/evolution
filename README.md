# Evolving Art - Work In Progress
A desktop app that lets you generate and evolve your own art in the form of pictures or videos.
This generates random expressions, and lets the user breed them. Images are evaluated using a stack machine with SIMD instructions, leveraging [SIMDeez](https://github.com/jackmott/simdeez)
and Rayon for parallelism at the instruction level and over multiple cores.

## Dependencies

```bash
sudo apt-get install build-essential cmake libclang1 libclang-common-11-dev
rustup install nightly
```

## running

```bash
export LD_LIBRARY_PATH=$(dirname $(locate libsleef.so.3 |grep release|head -n 1)
cargo run --bin evolution --features="ui"
```
## Documentation

### Command line input


```text
USAGE:
    evolution [OPTIONS]

OPTIONS:
    -h,  --help
            Print help information
    -V, --version
            Print version information
    -p, --pictures-path <PICTURES_PATH>
            The path to images that can be loaded via the Pic- operation [default: pictures]
    -t, --time <TIME>
            set the T variable (ms) [default: 0]
    -w, --width <WIDTH>
            The width of the generated image [default: 1920]
        --height <HEIGHT>
            The height of the generated image [default: 1080]
    -i, --input <INPUT>
            filename to read sexpr from and disabling the UI; Use '-' to read from stdin.
    -o, --output <OUTPUT>
            image file to write to
    -c, --copy-path <COPY_PATH>
            The path where to store a copy of the input and output files as part of the creative
            workflow
    -s, --coordinate-system <COORDINATE_SYSTEM>
            The Coordinate system to use [default: polar] [possible values: polar, cartesian]

```

* When the `--input` parameter is not set a Ui will start with several random generated examples.
* When the `--input` parameter is set, that will be used as a input.
* When the `--input` parameter is not "-" and the `--copy-path` parameter is set, the application will create a new image file if the input file changed. On success, the input and output files will be written (with timestamp prefix) in the copy-path directory.

### Animations
It is possible to create an animation by using the `--output` parameter:

```lisp
(RGB POLAR
	( X )
	( Y )
	(* T X )
)
```
![Animated Image](/samples/animation.gif)

- The `--output` parameter needs to be set to an animation filename (e.g. `.gif` extension).
- The source needs to contain at least 1 `T` Operation.
- When the `--time` parameter is set, the file will contain frames between t=0.0 and that end time.


### Ui mode

| Action | Select mode | Zoom Mode |
| ------ | ----------- | --------- |
| ESC               | Exit application                                                   |
| Spacebar          | Generate population                       | No-Op                  |
| Left mouse click  | Image and Sexpr files are written to disk                          |
| Right mouse click | Thumbnail is opened in Zoom mode          | Go back to select mode |

### Input DSL

The syntax for the input files are simple, case-insensitive, s-expressions.

```ebnf
SEXPR        = '(' PICTURE ')' ;
PICTURE      = MONO | GRAYSCALE | GRADIENT | RGB | HSV ;
MONO         = 'Mono' [COORDSYS] EXPR ;
RGB          = 'RGB' EXPR EXPR EXPR ;
HSV          = 'HSV' EXPR EXPR EXPR ;
GRAYSCALE    = 'Grayscale' EXPR ;
GRADIENT     = 'Gradient' '(' 'Colors' COLOR* ')' EXPR;
COORDSYS     = 'Polar' | 'Cartesian' | CHAR*;
COLORS       = '(' COLORTYPE EXPR EXPR EXPR ')' ;
COLORTYPE    = 'StopColor' | 'Color' ;
EXPR         = '(' EXPR ')';
             | '(' '+' EXPR ')' ;
             | '(' '-' EXPR ')' ;
             | '(' '*' EXPR ')' ;
             | '(' '/' EXPR ')' ;
             | '(' '%' EXPR ')' ;
             | '(' 'FBM' EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Ridge' EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Turbulence' EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Cell1' EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Cell2' EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'FBM3' EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'FBM4' EXPR EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Ridge3' EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Ridge4' EXPR EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Turbulence3' EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Turbulence4' EXPR EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Cell1_3' EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Cell1_4' EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Cell2_3' EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Cell2_4' EXPR EXPR EXPR EXPR EXPR EXPR EXPR ')' ;
             | '(' 'Mandelbrot' EXPR EXPR ')' ;
             | '(' 'Sin' EXPR ')' ;
             | '(' 'Tan' EXPR ')' ;
             | '(' 'Atan' EXPR ')' ;
             | '(' 'Atan2' EXPR EXPR ')' ;
             | '(' 'Min' EXPR ')' ;
             | '(' 'Max' EXPR ')' ;
             | '(' 'Square' EXPR ')' ;
             | '(' 'Wrap' EXPR ')' ;
             | '(' 'Clamp' EXPR ')' ;
             | '(' 'Ceil' EXPR ')' ;
             | '(' 'Floor' EXPR ')' ;
             | '(' 'Abs' EXPR ')' ;
             | '(' 'Log' EXPR ')' ;
             | '(' 'Sqrt' EXPR ')' ;
             | '(' 'Pic-' FILEDOTEXT EXPR EXPR ')';
             | 'WIDTH' ;
             | 'HEIGHT' ;
             | 'PI' ;
             | 'E' ;
             | 'x' ;
             | 'y' ;
             | 't' ;
             | CONSTANT ;
CONSTANT     = [NEGATE] DIGIT ;
             | [NEGATE] DIGIT* '.' DIGIT DIGIT* ;
DIGIT        = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9';
NEGATE       = '-';
FILEDOTEXT   = CHAR* '.' CHAR* ;
```

### Coordinate System

Invalid Coordinate systems are ignored, the default Coordinate System (Cartesian) will be used.

### Infinities and NaNs handling

* Positive infinity => + 1.0
* Negative infinity => - 1.0
* NaN               => 0.0

### Operations

### Constants PI, E, Width, Height

* `PI`: [std::f32::consts::PI](https://doc.rust-lang.org/nightly/std/f32/consts/constant.PI.html)
* `E`: [std::f32::consts::E](https://doc.rust-lang.org/nightly/std/f32/consts/constant.E.html)
* `WIDTH`: the `width` of the image; Either a default or set via the `--width` command line parameter.
* `HEIGHT`: the `height` of the image; Either a default or set via the `--height` command line parameter.

#### X, Y, T

* `X`: the `X` position in the image
* `Y`: the `Y` position in the image
* `T`: the frame id (milliseconds)

#### Ugh, Math...

The math operations (`+`, `-`, `*`, `/`, `%`, `sin`, `tan`, `atan`, `atan2`, `min`, `max`, `square`, `wrap`, `clap`, `ceil`, `floor`, `abs`, `log`, `sqrt`) work as expected [citation needed].

#### Noise

##### Fractal Brownian Motion (FBM)

* p0: Todo
* p1: Todo
* p2: Todo
* p3: Todo
* p4: Todo
* p5: Todo

##### Ridge

* p0: Todo
* p1: Todo
* p2: Todo
* p3: Todo
* p4: Todo
* p5: Todo

##### Turbulence

* p0: Todo
* p1: Todo
* p2: Todo
* p3: Todo
* p4: Todo
* p5: Todo

##### Cell1

* p0: Todo
* p1: Todo
* p2: Todo
* p3: Todo
* p4: Todo


##### Cell2

* p0: Todo
* p1: Todo
* p2: Todo
* p3: Todo
* p4: Todo

##### Multi-dimensional noise (3D / 4D)

Each noise family also has 3D and 4D variants that sample the noise field in
three or four dimensions — e.g. feed `X Y T` to animate noise over time. The
children are the coordinates, then a shared `freq` and `offset` that map each
axis as `coord * freq + offset` (so the sampling domain is expression-driven),
then the family-specific parameters:

* `FBM3` / `Ridge3` / `Turbulence3`: `x0 x1 x2 freq offset lacunarity gain`
* `FBM4` / `Ridge4` / `Turbulence4`: `x0 x1 x2 x3 freq offset lacunarity gain`
* `Cell1_3` / `Cell2_3`: `x0 x1 x2 freq offset jitter`
* `Cell1_4` / `Cell2_4`: `x0 x1 x2 x3 freq offset jitter`

Example: `( Mono ( FBM3 X Y T 0.5 0.0 2.0 0.5 ) )`.

##### Mandlebrot

This is not implemented yet.
Currently, it is a No-Op

* p0: Todo
* p1: Todo

## Some Possibilities

### HSV Images
```lisp
( HSV CARTESIAN
	( ( SQUARE ( / ( MANDELBROT X Y ) 0.7601185 ) ) )
	( ( + ( TAN ( TAN ( RIDGE Y ( ATAN -0.74197626 ) ( + Y Y ) Y ( CLAMP Y ) ( + X Y ) ) ) ) ( ATAN2 X Y ) ) )
	( ( MAX -0.9284358 Y ) )
)
 ```

![HSV Sample Image](/samples/hsv.png)

### Monochrome Images
```lisp
( MONO CARTESIAN
	( ( ATAN ( + ( CELL1 Y Y Y X ( - Y 0.7253959 ) ) ( ATAN X ) ) ) )
)
```

![Monochrome Sample Image](/samples/mono.png)

### Grayscale Images
```lisp
( GRAYSCALE POLAR
	( ( LOG ( + ( CELL1 ( LOG ( RIDGE ( SQRT Y ) Y Y X X 0.5701809 ) ) ( ATAN Y ) ( % Y 0.12452102 ) ( FLOOR ( ATAN2 Y Y ) ) ( SIN Y ) ) ( * ( + X ( SIN ( - ( ATAN2 Y X ) X ) ) ) ( ATAN ( LOG ( FLOOR ( SIN ( TURBULENCE Y 0.91551733 ( SQRT ( SQRT X ) ) ( MIN X Y ) -0.83923936 ( MANDELBROT Y X ) ) ) ) ) ) ) ) ) )
)
```

![Grayscale Sample Image](/samples/grayscale.png)

### RGB Images
```lisp
( RGB CARTESIAN
	( ( * ( TAN ( CLAMP ( ATAN ( SQRT ( MAX ( ABS ( FLOOR ( RIDGE 0.12349105 ( + X 0.500072 ) X X ( MAX Y Y ) 0.6249633 ) ) ) ( % ( CLAMP ( * Y ( SQUARE 0.39180493 ) ) ) ( WRAP ( CELL2 Y ( MIN -0.5756769 Y ) ( ABS 0.8329663 ) Y Y ) ) ) ) ) ) ) ) ( WRAP ( MANDELBROT ( SQRT ( TURBULENCE ( WRAP X ) 0.26766992 ( MANDELBROT -0.7147219 0.46446967 ) ( LOG 0.6340864 ) Y Y ) ) ( SQUARE ( * ( SIN ( / Y ( RIDGE X Y Y 0.49542284 X ( CEIL -0.7545812 ) ) ) ) ( CEIL ( TURBULENCE ( ATAN X ) X -0.52819157 -0.86907744 0.49089026 ( ATAN -0.5986686 ) ) ) ) ) ) ) ) )
	( ( / ( TURBULENCE ( FBM Y ( * ( RIDGE Y X X X X Y ) -0.98887086 ) 0.21490455 X X ( LOG X ) ) X ( % ( FLOOR X ) ( + X ( ATAN2 0.19268274 Y ) ) ) ( FBM Y -0.28251457 0.632663 X X X ) ( CEIL ( SQRT 0.8429725 ) ) ( WRAP ( MAX Y ( SQUARE ( TAN X ) ) ) ) ) ( FLOOR ( CELL1 ( + -0.5022187 ( LOG X ) ) ( RIDGE -0.8493159 Y ( TAN X ) Y Y Y ) ( ATAN ( SIN ( / ( ABS X ) ( CEIL 0.05049467 ) ) ) ) ( ATAN X ) ( TAN ( / ( FBM X X 0.802964 0.3002789 0.8905289 -0.06338668 ) ( SQUARE ( % X 0.48889422 ) ) ) ) ) ) ) )
	( ( ATAN ( SIN X ) ) )
)
```

![RGB Sample Image](/samples/rgb.png)


### Gradient Images
```lisp
( GRADIENT POLAR
	( COLORS
		( COLOR 0.38782334 0.18356442 0.5526812 )
		( COLOR 0.40132487 0.9418049 0.79687893 )
	)
	( FBM ( WRAP ( TAN ( - -0.90357685 ( ATAN Y ) ) ) ) ( ABS X ) ( ATAN2 Y X ) ( MAX Y ( MAX X X ) ) ( SQUARE ( CELL2 ( TAN Y ) Y Y X X ) ) ( * Y 0.009492159 ) )
)
 ```

![Gradient Sample Image](/samples/gradient.png)

