# eIsland – Änderungen (Russisch, kompaktes Widget, Verschieben/Fixieren, Ausblenden, Taskleiste)

Dateien in diesem ZIP liegen mit denselben Pfaden wie im Projekt – einfach über `eIsland-main/` kopieren.
Danach neu bauen: `npm install` und `npm run build` (Tauri, Rust-Toolchain nötig).
NICHT kompiliert und NICHT getestet (keine Build-Umgebung verfügbar).

## Verhalten
- Musik: Insel bleibt klein (idle), kein Songtext-Zustand, rotierende Randlinie entfernt. Schalter: `COMPACT_MUSIC_WIDGET` in `dynamicIslandStateConfig.ts`.
- Mausover vergrössert die Insel (forceClickToHover = false); Wegbewegen verkleinert.
- Ziehen auch im grossen Zustand (nicht auf Buttons/Reglern); Position wird nach dem Loslassen gespeichert (`window:island-position:save`).
- Begrenzung: nur oberste 120 DIP (`TOP_BAND` in layout.rs) und komplett im Bildschirm (Pill-Modus).
- Tray-Menü: «Скрыть на 10 с», «Скрыть на 15 с», «Закрепить положение» (Fixieren; wird auch im Hauptprozess erzwungen).
- Alle Fenster `skip_taskbar(true)`; Tray-Symbol bleibt.
- Russisch (`i18n/ru-RU.json`): komplette Oberfläche (ausser `devCli`), Standardsprache wenn keine andere gespeichert ist.
