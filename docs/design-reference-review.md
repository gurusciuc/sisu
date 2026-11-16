# Design reference review

## Current brief

The user selected monochrome black and white with neutral OKLCH lightness in 5% increments. This replaces the earlier sage accent direction. The browser remains a familiar functional tool with top tabs by default and optional left tabs. No marketing website is part of this milestone.

Visually inspected [Details' Heron collection](https://www.details.so/inspo/site/heronaiapp-com) and its [button example](https://www.details.so/inspo/heronaiapp-a814d6), linked to [Heron](https://heronaiapp.com/). The relevant qualities are flat neutral surfaces, fine outlines, clear spacing, strong sans-serif type and an explicit arrow affordance. Sisu uses an original interface and logo draft, without copying the reference assets.

The native Win32 palette maps `oklch(L% 0 0)` to 8-bit neutral sRGB. Dark surfaces use L15–30, light surfaces L95–100; text, borders, hover and focus use neutral tokens from the same scale. Quantization means adjacent low-lightness values can map to the same 8-bit value. The local new-tab page uses native CSS OKLCH tokens. Website content retains its own colors. All theme behavior still needs full acceptance, as listed in continue-tomorrow.md.

## Video

The supplied [TikTok](https://vt.tiktok.com/ZSpsuYyVP/) resolved to [Charles' interaction checklist](https://www.tiktok.com/@charles.unemployed/video/7704685504305876242) and was visually reviewed during playback. Its advice covers easing, short dropdown transitions, press feedback, trigger-connected popovers, interrupted movement and restrained motion in frequent interactions.

Applied or retained: immediate native feedback, matching nested corner shapes, reused live views during layout changes, no ornamental bounce for frequent tab actions, and a small local new-tab search-button press response with reduced-motion handling. Native menus use platform motion behavior. Rounded menu geometry has source code but still needs runtime acceptance. Stagger, blur, elastic dragging and shared-layout animation libraries were not added to this compiled browser shell. They are not needed to fix black flashes or stale pixels; those defects require correct view lifetime, presentation and invalidation.

## Logo draft

resources/branding/sisu.png is an AI-generated original black-and-white S mark draft. It is committed as a candidate asset. ICO/ICNS generation, small-size checks, executable/window branding and installer integration remain unfinished. No trademark clearance or final brand approval is implied.
