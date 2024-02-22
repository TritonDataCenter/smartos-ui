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
    console.log($element)
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
