/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2020 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2020 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */

hljs.registerLanguage("60", function (hljs) {
  const KEYWORDS = {
    keyword:
      'struct export import signal property animate for in if states transitions parent root self',
    literal:
      'true false',
    built_in:
      'Rectangle Image Text TouchArea Flickable Clip TextInput Window',
    type:
      'bool string int float length logical_length duration resource',
  };

  return {
    name: 'sixtyfps',
    aliases: ['60'],
    case_insensitive: false,
    keywords: KEYWORDS,
    contains: [
      hljs.QUOTE_STRING_MODE,
      hljs.C_LINE_COMMENT_MODE,
      hljs.C_BLOCK_COMMENT_MODE,
      hljs.COMMENT('/\\*', '\\*/', {
        contains: ['self']
      }),
      {
        className: 'number',
        begin: '\\b\\d+(\\.\\d+)?(\\w+)?',
        relevance: 0
      },
      {
        className: 'title',
        begin: '\\b[_a-zA-Z][_\\-a-zA-Z0-9]* *:=',
      },
      {
        className: 'symbol',
        begin: '\\b[_a-zA-Z][_\\-a-zA-Z0-9]*(:| *=>)',
      },
      {
        className: 'built_in',
        begin: '\\b[_a-zA-Z][_\\-a-zA-Z0-9]*!',
      },
    ],
    illegal: /@/
  };
});

document
  .querySelectorAll("code.language-60")
  .forEach((block) => hljs.highlightBlock(block));