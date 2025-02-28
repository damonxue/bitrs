import '@/styles/globals.css';
import type { AppProps } from 'next/app';
import { ChakraProvider, extendTheme } from '@chakra-ui/react';
import { QueryClient, QueryClientProvider } from 'react-query';
import Layout from '@/components/layout/Layout';
import { WalletProvider } from '@/context/WalletContext';

// 创建React Query客户端
const queryClient = new QueryClient();

// 自定义主题
const theme = extendTheme({
  colors: {
    brand: {
      900: '#1a365d',
      800: '#153e75',
      700: '#2a69ac',
    },
    bg: {
      primary: '#0f172a',
      secondary: '#1e293b',
      hover: '#334155',
      card: '#1e293b',
    },
    text: {
      primary: '#f8fafc',
      secondary: '#94a3b8',
      accent: '#38bdf8',
    },
    action: {
      buy: '#10b981',
      sell: '#ef4444',
      button: '#3b82f6',
      buttonHover: '#2563eb',
    },
  },
  styles: {
    global: {
      body: {
        bg: 'bg.primary',
        color: 'text.primary',
      },
    },
  },
});

export default function App({ Component, pageProps }: AppProps) {
  return (
    <QueryClientProvider client={queryClient}>
      <ChakraProvider theme={theme}>
        <WalletProvider>
          <Layout>
            <Component {...pageProps} />
          </Layout>
        </WalletProvider>
      </ChakraProvider>
    </QueryClientProvider>
  );
}