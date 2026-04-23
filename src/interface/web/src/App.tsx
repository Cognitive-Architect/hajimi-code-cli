import { Layout } from './components/Layout'
import { Pane } from './components/Pane'
import { TypeRacingWidget } from './components/TypeRacingWidget'

function App() {
  return (
    <Layout>
      <Pane title="Explorer" defaultWidth={250} resizable>
        <div>File Explorer</div>
      </Pane>
      <Pane title="Editor" resizable={false}>
        <div className="editor-container">
          <div className="editor-content">Editor Content</div>
          <TypeRacingWidget
            uri="file:///demo.rs"
            line={10}
            character={5}
            code="let x = 42;"
            debounceMs={300}
          />
        </div>
      </Pane>
      <Pane title="Preview" defaultWidth={300} resizable collapsible>
        <div>Preview Panel</div>
      </Pane>
    </Layout>
  )
}

export default App
