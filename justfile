# just configuration file

# set Powershell for Windows
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# dev server
dev:
    cd frontend; trunk serve --open
