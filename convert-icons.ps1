$magick = "C:\Program Files\ImageMagick-7.1.2-Q16-HDRI\magick.exe"
$iconDir = "src-tauri\icons"
$sourceIcon = "$iconDir\app-icon.png"

# Convert to ICO with multiple resolutions
& $magick $sourceIcon -define icon:auto-resize=256,128,64,48,32,16 "$iconDir\icon.ico"

# Generate PNG files at different sizes
& $magick $sourceIcon -resize 128x128 "$iconDir\128x128.png"
& $magick $sourceIcon -resize 256x256 "$iconDir\128x128@2x.png"
& $magick $sourceIcon -resize 32x32 "$iconDir\32x32.png"

# Copy as main icon.png
Copy-Item -Path $sourceIcon -Destination "$iconDir\icon.png" -Force

Write-Host "Icon conversion completed successfully!" -ForegroundColor Green
