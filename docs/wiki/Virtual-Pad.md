# Virtual pad (web, phones and tablets)

The web build shows an on-screen gamepad on every touch device: a stick on the left, eight attack buttons (`X Y Z SP` / `A B C SUPER`) and a row with `BACK`, `OK`, `AWAKE` and `II` (pause) on the right. It is drawn with plain HTML and CSS shapes (no sprites) in `web/vpad.js`, above the canvas, and stays visible on every screen: title, menus, select, VS, fight, pause and story.

## How it works

- Each finger is tracked by its `pointerId`, so the stick and several buttons can be held at the same time. A finger can slide from one attack button to the next.
- The browser hit-tests the buttons itself, so canvas size, device pixel ratio and render scale do not affect where a touch lands.
- The engine reads the pad once per frame through `ume_vpad_poll()` (fighter bits) and `ume_vpad_nav()` (menu `OK`/`BACK`/`START`). The stick and `A`/`B` also drive menus, like a physical gamepad.
- A tap shorter than one frame is still delivered for two frames.
- Fingers are released on `pointerup`, `pointercancel`, `lostpointercapture`, page blur, tab hide and `pagehide`. A long press (`contextmenu`) does not release them.
- The pad turns on when the device reports touch (`maxTouchPoints`, `ontouchstart`, `pointer: coarse`) or on the first touch. Nothing hides it afterwards. `?touch=1` forces it on a desktop.

## Debugging on a real phone

Open the game with `?debug=1`. A box in the top-left corner shows whether the pad is on, the device pixel ratio, the viewport, the fingers held, the bits sent to the engine and counts of `pointerdown`, `pointerup` and `pointercancel`.

## Tests

`node scripts/web-vpad-smoke.js web/dist` drives the real web build in Chromium with real multi-touch (`Input.dispatchTouchEvent`). It checks that the pad is visible and unobstructed on every screen, navigates the menu, select and stage screens only by touch, walks, jumps, attacks while holding the stick, holds three fingers at once, releases one finger without losing the stick, pauses and resumes. `VPAD_DEVICE` picks the Playwright device (default `Pixel 7 landscape`; CI also runs `Pixel 7` portrait and `iPhone 13 landscape`). `VPAD_SHOTS=<dir>` saves screenshots.
