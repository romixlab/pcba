# pcba
> Opinionated PCB design CLI helper tool. Mainly focused on KiCad development, but not entirely.

The goal is to keep this tool simple and easy to install and use. No external dependencies apart from KiCad itself.

## Conventions

* PCB project folder name: `b<nnn><rev>_short_description`, e.g., `b123a_dc_dc_module`.
  * All subfolders and files should not contain short description to simplify renaming if need be.
  * I.e., `board_name` itself is `b<nnn><rev>`.
* Date format: `<dd><mmm><yyyy>`, with leading 0 on day number, e.g., `05nov2025`, to avoid any confusion.
* File names must contain a prefix: `<prefix>_<board_name>`, e.g., `errata_b123a.md`, to facilitate searching and provide clear meaning when multiple files are open.
* Lower case used for all file and folder names, for no other reason, but making rules less ambiguous.

## Features:

* Create a folder structure for KiCad project
  * [ ] Simplified
  * [ ] Full
* Find board in a folder with many projects
  * [ ] By board serial number
  * [ ] By MPN sub-string that is used on a board (e.g. STM32G0)

