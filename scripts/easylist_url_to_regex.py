import sys
import pathlib
# https://github.com/scrapinghub/adblockparser
from adblockparser import AdblockRule

if __name__ == '__main__':
    if len(sys.argv) == 1 or not pathlib.Path(sys.argv[1]).exists():
        print("Please provide the path to an existing file.")
        exit(1)

    with open(sys.argv[1]) as file:
        for line in file:
            # Stop after the comment sectioning off element hiding
            #if "General element hiding rules" in line:
            #    break

            rule = AdblockRule(line)
            if rule.is_comment or rule.is_exception or rule.is_html_rule:
                continue
            else:
                print(rule.regex)

