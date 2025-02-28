import { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  GridItem,
  Flex,
  Heading,
  Select,
  useToast,
} from '@chakra-ui/react';
import Head from 'next/head';
import { useWallet } from '@solana/wallet-adapter-react';
import OrderBook from '@/components/trading/OrderBook';
import TradingChart from '@/components/trading/TradingChart';
import TradeForm from '@/components/trading/TradeForm';
import RecentTrades from '@/components/trading/RecentTrades';
import MarketSelector from '@/components/trading/MarketSelector';
import WalletRequiredAlert from '@/components/common/WalletRequiredAlert';
import { Market } from '@/types/market';
import { useMarketData } from '@/hooks/useMarketData';

export default function Home() {
  const { connected } = useWallet();
  const toast = useToast();
  
  // 获取市场列表
  const { data: markets, isLoading: marketsLoading, error: marketsError } = useMarketData();
  
  // 当前选中的市场
  const [selectedMarket, setSelectedMarket] = useState<Market | null>(null);
  
  // 当市场数据加载完成后，设置默认选中的市场
  useEffect(() => {
    if (markets && markets.length > 0 && !selectedMarket) {
      setSelectedMarket(markets[0]);
    }
  }, [markets, selectedMarket]);

  // 显示错误信息
  useEffect(() => {
    if (marketsError) {
      toast({
        title: '加载市场数据失败',
        description: '无法连接到交易服务器，请稍后再试',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    }
  }, [marketsError, toast]);

  // 处理市场切换
  const handleMarketChange = (marketId: string) => {
    if (markets) {
      const market = markets.find(m => m.market_id === marketId);
      if (market) {
        setSelectedMarket(market);
      }
    }
  };

  return (
    <>
      <Head>
        <title>BitRS DEX | 分布式交易所</title>
        <meta name="description" content="基于Solana的高性能分布式交易所" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Box mb={6}>
        <Flex justify="space-between" align="center" mb={4}>
          <Heading size="md" color="text.primary">
            交易所
          </Heading>
          
          {markets && (
            <MarketSelector 
              markets={markets} 
              selectedMarketId={selectedMarket?.market_id || ''} 
              onChange={handleMarketChange}
              isLoading={marketsLoading}
            />
          )}
        </Flex>

        {!connected && <WalletRequiredAlert mb={4} />}

        <Grid
          templateColumns={{ base: 'repeat(1, 1fr)', md: 'repeat(3, 1fr)' }}
          templateRows={{ base: 'repeat(4, auto)', md: 'repeat(2, auto)' }}
          gap={4}
        >
          {/* 图表区域 - 在移动设备上占据整行，在桌面上占据2/3 */}
          <GridItem colSpan={{ base: 1, md: 2 }} rowSpan={1}>
            <TradingChart market={selectedMarket} />
          </GridItem>

          {/* 订单簿 - 在移动设备上占据整行，在桌面上占据1/3 */}
          <GridItem colSpan={{ base: 1, md: 1 }} rowSpan={2}>
            <OrderBook market={selectedMarket} />
          </GridItem>

          {/* 交易表单 - 在移动设备上占据整行，在桌面上占据2/3 */}
          <GridItem colSpan={{ base: 1, md: 2 }} rowSpan={1}>
            <TradeForm market={selectedMarket} disabled={!connected} />
          </GridItem>

          {/* 最近交易记录 - 在移动设备上占据整行 */}
          <GridItem colSpan={{ base: 1, md: 3 }} rowSpan={1}>
            <RecentTrades market={selectedMarket} />
          </GridItem>
        </Grid>
      </Box>
    </>
  );
}