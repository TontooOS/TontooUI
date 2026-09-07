#!/bin/bash
# Toggle GNOME GTK theme between Dark (#1d1d1d) and Light (#ececec)
# Works for TontooUI ColorScheme::detect_system() — toggles gtk-theme-name and prefer-dark

set -e

THEME=$(gsettings get org.gnome.desktop.interface gtk-theme 2>/dev/null || echo "'Adwaita'")
SCHEME=$(gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null || echo "'default'")

is_dark=false
if [[ "$THEME" == *"dark"* ]] || [[ "$THEME" == *"Dark"* ]]; then
  is_dark=true
fi
if [[ "$SCHEME" == *"dark"* ]] || [[ "$SCHEME" == *"prefer-dark"* ]]; then
  is_dark=true
fi

if [[ "$is_dark" == true ]]; then
  echo "Switching to Light..."
  gsettings set org.gnome.desktop.interface gtk-theme 'Adwaita' 2>/dev/null || true
  gsettings set org.gnome.desktop.interface color-scheme 'prefer-light' 2>/dev/null || gsettings set org.gnome.desktop.interface color-scheme 'default' 2>/dev/null || true
  # GTK3/4 settings.ini
  mkdir -p ~/.config/gtk-3.0 ~/.config/gtk-4.0
  for f in ~/.config/gtk-3.0/settings.ini ~/.config/gtk-4.0/settings.ini; do
    if [ -f "$f" ]; then
      sed -i 's/^gtk-application-prefer-dark-theme=.*/gtk-application-prefer-dark-theme=false/' "$f" 2>/dev/null || true
      sed -i 's/^gtk-theme-name=.*/gtk-theme-name=Adwaita/' "$f" 2>/dev/null || true
      grep -q "gtk-application-prefer-dark-theme" "$f" || echo "gtk-application-prefer-dark-theme=false" >> "$f"
      grep -q "gtk-theme-name" "$f" || echo "gtk-theme-name=Adwaita" >> "$f"
    else
      printf "[Settings]\ngtk-theme-name=Adwaita\ngtk-application-prefer-dark-theme=false\n" > "$f"
    fi
  done
  echo "Light (#ececec) active"
else
  echo "Switching to Dark..."
  gsettings set org.gnome.desktop.interface gtk-theme 'Adwaita-dark' 2>/dev/null || true
  gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark' 2>/dev/null || true
  mkdir -p ~/.config/gtk-3.0 ~/.config/gtk-4.0
  for f in ~/.config/gtk-3.0/settings.ini ~/.config/gtk-4.0/settings.ini; do
    if [ -f "$f" ]; then
      sed -i 's/^gtk-application-prefer-dark-theme=.*/gtk-application-prefer-dark-theme=true/' "$f" 2>/dev/null || true
      sed -i 's/^gtk-theme-name=.*/gtk-theme-name=Adwaita-dark/' "$f" 2>/dev/null || true
      grep -q "gtk-application-prefer-dark-theme" "$f" || echo "gtk-application-prefer-dark-theme=true" >> "$f"
      grep -q "gtk-theme-name" "$f" || echo "gtk-theme-name=Adwaita-dark" >> "$f"
    else
      printf "[Settings]\ngtk-theme-name=Adwaita-dark\ngtk-application-prefer-dark-theme=true\n" > "$f"
    fi
  done
  echo "Dark (#1d1d1d) active"
fi

# Verify for TontooUI
echo "gtk-theme: $(gsettings get org.gnome.desktop.interface gtk-theme 2>/dev/null)"
echo "color-scheme: $(gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null)"
