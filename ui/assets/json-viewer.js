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
    editor.dispatch({
      changes: {
        from: 0,
        to: editor.state.doc.length,
        insert: $($element.dataset.content).value
      }
    })
  })
}
