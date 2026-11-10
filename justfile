run:
  just clean || true
  cargo run --release

view:
  open output/frame_000000000.png

video filename:
  just clean || true
  cargo run --release
  just convert {{filename}}

convert filename='output':
  ffmpeg -framerate 24 -pattern_type glob -i './output/frame_*.png' -c:v libx264 -pix_fmt yuv420p {{filename}}.mp4

clean:
  rm ./output/frame_*.png
