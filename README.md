diff --git a/README.md b/README.md
new file mode 100644
index 0000000000000000000000000000000000000000..c14be136388f03944d0bdf66d9db948395203a44
--- /dev/null
+++ b/README.md
@@ -0,0 +1,28 @@
+# Itadaki Street (Fortune Street) Rust Prototype
+
+This project is a lightweight Bevy prototype of the Wii-era *Fortune Street* (Itadaki Street) gameplay loop. It focuses on a 2D board, UI overlays, and basic bot turns so you can explore the core mechanics before adding full content.
+
+## Implemented rules (Fortune Street basics)
+- Players roll a die, move along a loop of shops, and resolve the tile they land on.
+- Shops can be bought if unowned; landing on another player's shop pays a fee.
+- Four suit tiles (♠ ♥ ♦ ♣) must be collected before visiting the bank to level up and collect a salary based on net worth.
+- Chance tiles give small cash bonuses or penalties.
+- A district/stocks concept exists: districts track shop counts and each player holds stock balances for later expansion of the economy.
+
+## Controls and UI
+- **Camera pan:** Arrow keys or WASD
+- **Zoom:** Mouse wheel scroll
+- **Toggle main menu:** `M` (shows fast decision and management options)
+- **Toggle stocks menu:** `S` (opens detailed stock panel; also auto-opens the main menu)
+- The left sidebar lists each player's cash, net worth, level, suits, properties owned, and stocks. The current turn is highlighted.
+
+## Running
+```
+cargo run
+```
+
+## Roadmap ideas
+- Human interaction for buying, auctioning, and stock trading
+- Full chance card deck, auctions, and shop upgrades
+- Saving/loading board definitions for different maps
+- Improved art and animation
