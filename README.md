# Nifty Options data fetcher

1. Hit API request and fetch data 
2. Filter relevant timeframe data  
3. Process data to generate OI for PUT and CALL for every segment. 
4. Additional rule implementation for generating triggers.
    a. Maintain highest OI for the day for PUT and CALL.
    b. If the subsequent OI is less than 10% of the highest, trigger console message.
5. Perform all above steps every minute.


# Uncomment the println lines to print relevant logs.

# Release build for windows : 
# Install dependencies :
brew install mingw-w64
brew install gcc-mingw-w64-x86-64
brew install sdl2
# Build command
cargo build --target=x86_64-pc-windows-gnu --verbose --release