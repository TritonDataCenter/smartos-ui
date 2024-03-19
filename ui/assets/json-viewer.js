/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

import { $, $$ } from './global'
import {
  EditorView,
  EditorState,
  basicSetup,
  json,
  oneDark
} from '@tpaul/codemirror6-json-rolledup'

// Setup a read-only CodeMirror instance for JSON viewing
export const setupJSONViewer = () => {
  $$('.json-viewer').forEach($element => {
    const extensions = [
      basicSetup,
      json(),
      oneDark,
      EditorState.readOnly.of(true)
    ]
    const editor = new EditorView({ extensions, parent: $element })

    // Parse the JSON so we can print it formatted
    let text = $($element.dataset.content).value
    try {
      text = JSON.stringify(JSON.parse(text), null, 2)
    } catch (e) {
      window.alert('Failed parsing JSON')
    }

    editor.dispatch({
      changes: {
        from: 0,
        to: editor.state.doc.length,
        insert: text
      }
    })
  })
}
