@echo off
if not exist "node_modules" (
    echo node_modules not found. Installing Tailwind dependencies...
    call npm install
)

echo Starting Tailwind CSS Watch process...
start "Tailwind Watch" cmd /k "npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch"

echo Starting Dioxus App (dx serve)...
dx serve
