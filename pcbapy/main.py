import os

from pygerber.gerberx3.api.v2 import GerberFile, Project, FileTypeEnum, PixelFormatEnum
import argparse
import re


def find_project(pcb_number):
    hw_dir = "/home/roman/git/hw/"
    for folder_name in os.listdir(hw_dir):
        if folder_name.startswith(f"b{pcb_number}"):
            return hw_dir + folder_name + "/"
    raise ValueError(f"Project 'b{pcb_number}' not found in '{hw_dir}'")


def parse_pcb_string(s: str):
    """
    Parse string in format: bnnnr (e.g., b123a)
    Returns (pcb_number, pcb_revision) if valid, else raises ValueError
    """
    # Strip whitespace
    s = s.strip()

    # Regex: starts with 'b', followed by 1 or more digits, ends with 1 letter
    pattern = r'^b(\d+)([a-z])$'

    match = re.match(pattern, s)

    if not match:
        raise ValueError(
            f"Invalid PCB format: '{s}'. Expected format: bnnnr (e.g., b123a)"
        )

    pcb_number = int(match.group(1))
    pcb_revision = match.group(2)

    return pcb_number, pcb_revision


def main(args):
    pcb_number, pcb_rev = parse_pcb_string(args.project)
    pcba_dir = find_project(pcb_number)
    pcba_with_rev = f"b{pcb_number}{pcb_rev}"
    gerber_base = pcba_dir + "/gerbers/" + pcba_with_rev + "/" + pcba_with_rev + "-"

    project = Project([
        GerberFile.from_file(gerber_base + "F_Cu.gtl", FileTypeEnum.COPPER),
        GerberFile.from_file(gerber_base + "F_Mask.gts", FileTypeEnum.MASK),
        # GerberFile.from_file(gerber_base + "F_Paste.gtp", FileTypeEnum.PASTE),
        GerberFile.from_file(gerber_base + "F_Silkscreen.gto", FileTypeEnum.SILK),
    ])

    project.parse().render_raster(pcba_dir + "render/front.png", dpmm=100, pixel_format=PixelFormatEnum.RGBA)

    # GerberFile.from_file(gerber_base + "F_Cu.gtl").parse().render_raster(
    #    "output.png",
    #    dpmm=100,
    #    color_scheme=ColorScheme.COPPER_ALPHA,
    #    pixel_format=PixelFormatEnum.RGBA,
    # )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="PCB helper tool")
    parser.add_argument("project", help="Name of the project in hw folder")
    # parser.add_argument("-r", "--revision", type=str, default="a", help="PCBA revision")
    args = parser.parse_args()
    main(args)
