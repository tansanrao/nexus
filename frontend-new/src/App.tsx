import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ThemeProvider } from './contexts/ThemeProvider';
import { ApiConfigProvider } from './contexts/ApiConfigContext';
import { CodeThemeProvider } from './contexts/CodeThemeContext';
import { TimezoneProvider } from './contexts/TimezoneContext';
import { ThreadBrowser } from './pages/ThreadBrowser';
import { SettingsPage } from './pages/Settings';
import { SettingsGeneral } from './pages/SettingsGeneral';
import { SettingsDatabase } from './pages/SettingsDatabase';
import { SettingsSystemStatistics } from './pages/SettingsSystemStatistics';

function App() {
  return (
    <ThemeProvider>
      <ApiConfigProvider>
        <TimezoneProvider>
          <CodeThemeProvider>
            <BrowserRouter>
              <Routes>
                <Route path="/" element={<ThreadBrowser />} />
                <Route path="/settings" element={<SettingsPage />}>
                  <Route index element={<Navigate to="general" replace />} />
                  <Route path="general" element={<SettingsGeneral />} />
                  <Route path="database" element={<SettingsDatabase />} />
                  <Route path="system" element={<SettingsSystemStatistics />} />
                </Route>
              </Routes>
            </BrowserRouter>
          </CodeThemeProvider>
        </TimezoneProvider>
      </ApiConfigProvider>
    </ThemeProvider>
  );
}

export default App;
