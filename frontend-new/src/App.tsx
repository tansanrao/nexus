import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ThemeProvider } from './contexts/ThemeProvider';
import { ApiConfigProvider } from './contexts/ApiConfigContext';
import { ThreadBrowser } from './pages/ThreadBrowser';

function App() {
  return (
    <ThemeProvider 
      attribute="class" 
      defaultTheme="light" 
      enableSystem
      themes={['light', 'dark', 'hackernews', 'solarized-light', 'solarized-dark']}
    >
      <ApiConfigProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<ThreadBrowser />} />
          </Routes>
        </BrowserRouter>
      </ApiConfigProvider>
    </ThemeProvider>
  );
}

export default App;
