import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AppLayout } from './components/AppLayout';
import { ThreadBrowser } from './pages/ThreadBrowser';
import { AuthorBrowser } from './pages/AuthorBrowser';
import { Settings } from './pages/Settings';
import { SettingsGeneral } from './pages/SettingsGeneral';
import { SettingsDatabase } from './pages/SettingsDatabase';
import { SettingsSystemStatistics } from './pages/SettingsSystemStatistics';
import { ThemeProvider } from './contexts/ThemeContext';
import { TimezoneProvider } from './contexts/TimezoneContext';
import { MailingListProvider } from './contexts/MailingListContext';
import { ApiConfigProvider } from './contexts/ApiConfigContext';

function App() {
  return (
    <ApiConfigProvider>
      <ThemeProvider>
        <TimezoneProvider>
          <MailingListProvider>
            <Router>
              <AppLayout>
                <Routes>
                  {/* Redirect root to threads */}
                  <Route path="/" element={<Navigate to="/threads" replace />} />

                  {/* Thread routes - static paths */}
                  <Route path="/threads" element={<ThreadBrowser />} />
                  <Route path="/threads/:threadId" element={<ThreadBrowser />} />

                  {/* Author routes - static paths */}
                  <Route path="/authors" element={<AuthorBrowser />} />
                  <Route path="/authors/:authorId" element={<AuthorBrowser />} />
                  <Route path="/authors/:authorId/threads/:threadId" element={<AuthorBrowser />} />

                  {/* Settings routes */}
                  <Route path="/settings" element={<Settings />}>
                    <Route index element={<Navigate to="/settings/general" replace />} />
                    <Route path="general" element={<SettingsGeneral />} />
                    <Route path="database" element={<SettingsDatabase />} />
                    <Route path="system" element={<SettingsSystemStatistics />} />
                  </Route>
                </Routes>
              </AppLayout>
            </Router>
          </MailingListProvider>
        </TimezoneProvider>
      </ThemeProvider>
    </ApiConfigProvider>
  );
}

export default App;
