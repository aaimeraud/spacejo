# Image de production de SpaceJo : l'interface et le serveur Rust dans un seul conteneur (Render).
# Trois étapes : seule la dernière, minimale, part en production.

# --- 1. Interface (React + Vite) ---
FROM node:24-slim AS web
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
# Adresse publique du site, pour les aperçus des liens partagés (Render passe ses variables
# d'environnement à la construction de l'image).
ARG VITE_SITE_URL=""
ENV VITE_SITE_URL=${VITE_SITE_URL}
# Construit, puis précompresse les fichiers texte : le serveur (tower-http) envoie la version .gz aux
# navigateurs qui l'acceptent (environ trois fois plus légère pour le JavaScript).
RUN npm run build \
  && find dist -type f \( -name '*.js' -o -name '*.css' -o -name '*.html' -o -name '*.svg' \
       -o -name '*.webmanifest' -o -name '*.ico' \) -exec gzip -9 -k {} +

# --- 2. Serveur (Rust) ---
FROM rust:1.98.0-slim-trixie AS server
WORKDIR /app
# D'abord les seules dépendances, avec des sources vides : cette couche reste en cache tant que
# Cargo.toml et Cargo.lock ne changent pas, et les builds suivants ne recompilent que le projet.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/spacejo-core/Cargo.toml crates/spacejo-core/
RUN mkdir -p src/bin crates/spacejo-core/src \
  && touch src/lib.rs crates/spacejo-core/src/lib.rs \
  && echo 'fn main() {}' > src/bin/server.rs \
  && echo 'fn main() {}' > src/bin/migrate.rs \
  && cargo build --release --locked --bin server \
  && rm -rf src crates/spacejo-core/src
# Puis le vrai code (les migrations SQL sont intégrées au binaire par `sqlx::migrate!`).
COPY crates crates
COPY src src
COPY migrations migrations
RUN touch src/lib.rs crates/spacejo-core/src/lib.rs \
  && cargo build --release --locked --bin server

# --- 3. Image finale ---
FROM debian:trixie-slim
# Pas de certificats système à installer : la connexion TLS à PostgreSQL embarque ses propres racines
# (rustls + webpki-roots). Le serveur tourne sous un utilisateur sans droits.
RUN useradd --system --uid 10001 --no-create-home spacejo
WORKDIR /app
COPY --from=server /app/target/release/server /app/server
COPY --from=web /app/web/dist /app/web/dist
ENV STATIC_DIR=/app/web/dist \
    PORT=10000
USER spacejo
EXPOSE 10000
CMD ["/app/server"]
