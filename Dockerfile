# Stage 1: Build all games with Rust + Trunk
FROM rust:1.82-bookworm AS builder

# Install trunk and wasm target
RUN cargo install trunk --version 0.21.5 && \
    rustup target add wasm32-unknown-unknown

WORKDIR /build

# Copy all game directories
COPY 2048 ./2048
COPY generic-multi-sudoku ./generic-multi-sudoku
COPY hex-puzzle ./hex-puzzle
COPY merge-game ./merge-game
COPY multi-sudoku ./multi-sudoku
COPY number-match ./number-match
COPY sudoku ./sudoku
COPY water-sort ./water-sort

# Build each game with trunk
RUN cd /build/2048 && trunk build --release
RUN cd /build/generic-multi-sudoku && trunk build --release
RUN cd /build/hex-puzzle && trunk build --release
RUN cd /build/merge-game && trunk build --release
RUN cd /build/multi-sudoku && trunk build --release
RUN cd /build/number-match && trunk build --release
RUN cd /build/sudoku && trunk build --release
RUN cd /build/water-sort && trunk build --release

# Stage 2: Serve with nginx
FROM nginx:alpine

# Copy built game assets to nginx
COPY --from=builder /build/2048/dist /usr/share/nginx/games/2048
COPY --from=builder /build/generic-multi-sudoku/dist /usr/share/nginx/games/generic-multi-sudoku
COPY --from=builder /build/hex-puzzle/dist /usr/share/nginx/games/hex-puzzle
COPY --from=builder /build/merge-game/dist /usr/share/nginx/games/merge-game
COPY --from=builder /build/multi-sudoku/dist /usr/share/nginx/games/multi-sudoku
COPY --from=builder /build/number-match/dist /usr/share/nginx/games/number-match
COPY --from=builder /build/sudoku/dist /usr/share/nginx/games/sudoku
COPY --from=builder /build/water-sort/dist /usr/share/nginx/games/water-sort

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 8001 8002 8003 8004 8005 8006 8007 8008

CMD ["nginx", "-g", "daemon off;"]
