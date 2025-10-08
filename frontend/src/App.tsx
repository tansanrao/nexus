import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AppLayout } from './components/AppLayout';
import { ThreadBrowser } from './pages/ThreadBrowser';
import { AuthorBrowser } from './pages/AuthorBrowser';
import { Settings } from './pages/Settings';
import { TimezoneProvider } from './contexts/TimezoneContext';

function App() {
  return (
    <TimezoneProvider>
      <Router>
        <AppLayout>
          <Routes>
            {/* Redirect root to default mailing list */}
            <Route path="/" element={<Navigate to="/bpf/threads" replace />} />

            {/* Thread routes - combined list and view */}
            <Route path="/:mailingList/threads" element={<ThreadBrowser />} />
            <Route path="/:mailingList/threads/:threadId" element={<ThreadBrowser />} />

            {/* Author routes - three column layout */}
            <Route path="/:mailingList/authors" element={<AuthorBrowser />} />
            <Route path="/:mailingList/authors/:authorId" element={<AuthorBrowser />} />
            <Route path="/:mailingList/authors/:authorId/threads/:threadId" element={<AuthorBrowser />} />

            {/* Settings */}
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </AppLayout>
      </Router>
    </TimezoneProvider>
  );
}

export default App;
