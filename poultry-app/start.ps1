if (-not (Test-Path "node_modules")) {
    Write-Host "node_modules not found. Installing Tailwind dependencies..." -ForegroundColor Yellow
    cmd /c npm install
}

# Launch Tailwind CSS CLI watch process in a separate window
Start-Process cmd -ArgumentList "/k npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch"

# Run Dioxus serve in current shell session
dx serve
