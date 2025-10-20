import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ThemeProvider } from './contexts/ThemeProvider';
import { ApiConfigProvider } from './contexts/ApiConfigContext';
import { ThreadBrowser } from './pages/ThreadBrowser';
import { CodeThemeProvider } from './contexts/CodeThemeContext';

function App() {
  return (
    <ThemeProvider 
      attribute="class" 
      defaultTheme="light" 
      enableSystem
      themes={['light', 'dark', 'solarized-light', 'solarized-dark']}
    >
      <ApiConfigProvider>
        <CodeThemeProvider>
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<ThreadBrowser />} />
            </Routes>
          </BrowserRouter>
        </CodeThemeProvider>
      </ApiConfigProvider>
    </ThemeProvider>
  );
}

export default App;
