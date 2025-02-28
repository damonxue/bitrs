import React, { useEffect, useRef } from 'react';
import { Box, Center, Spinner, Text } from '@chakra-ui/react';
import { createChart, ColorType, IChartApi, ISeriesApi, CandlestickData } from 'lightweight-charts';
import { Market } from '@/types/market';

interface TradingChartProps {
  market: Market | null;
}

const TradingChart: React.FC<TradingChartProps> = ({ market }) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);

  // 初始化和更新图表
  useEffect(() => {
    if (!chartContainerRef.current) return;

    if (!chartRef.current) {
      // 创建图表实例
      const chart = createChart(chartContainerRef.current, {
        width: chartContainerRef.current.clientWidth,
        height: 400,
        layout: {
          background: { color: '#1e293b' },
          textColor: '#94a3b8',
        },
        grid: {
          vertLines: { color: '#334155' },
          horzLines: { color: '#334155' },
        },
        timeScale: {
          borderColor: '#334155',
        },
        crosshair: {
          mode: 0, // 0表示正常模式，1表示磁性模式
        },
        localization: {
          locale: 'zh-CN',
        },
        rightPriceScale: {
          borderColor: '#334155',
        },
      });

      // 创建K线图系列
      const candleSeries = chart.addCandlestickSeries({
        upColor: '#10b981',
        downColor: '#ef4444',
        borderVisible: false,
        wickUpColor: '#10b981',
        wickDownColor: '#ef4444',
      });

      // 保存引用
      chartRef.current = chart;
      seriesRef.current = candleSeries;
    }

    // 调整图表大小的函数
    const handleResize = () => {
      if (chartRef.current && chartContainerRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    // 监听窗口大小变化
    window.addEventListener('resize', handleResize);

    // 清理函数
    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  // 加载市场数据
  useEffect(() => {
    if (!market || !seriesRef.current) return;

    // 在实际应用中，这里应该从API获取K线数据
    // 以下是模拟数据，展示图表功能

    // 生成近30天的模拟K线数据
    const currentDate = new Date();
    const data: CandlestickData[] = [];
    
    // 基础价格，根据市场不同设置不同的基础价格
    let basePrice = market.market_id.includes('BTC') ? 50000 : 
                    market.market_id.includes('SOL') ? 50 : 100;
    
    // 生成模拟数据
    for (let i = 29; i >= 0; i--) {
      const date = new Date();
      date.setDate(currentDate.getDate() - i);
      
      // 随机波动
      const volatility = basePrice * 0.03; // 3%波动
      const open = basePrice * (1 + (Math.random() * 0.02 - 0.01));
      const close = open * (1 + (Math.random() * 0.04 - 0.02));
      const high = Math.max(open, close) * (1 + Math.random() * 0.01);
      const low = Math.min(open, close) * (1 - Math.random() * 0.01);
      
      data.push({
        time: Math.floor(date.getTime() / 1000) as any,
        open,
        high,
        low,
        close,
      });
      
      // 更新基础价格以创造趋势
      basePrice = close;
    }

    // 设置数据
    seriesRef.current.setData(data);

    // 确保所有数据都可见
    chartRef.current?.timeScale().fitContent();
  }, [market]);

  if (!market) {
    return (
      <Box bg="bg.card" borderRadius="md" p={4} height="400px">
        <Center h="100%">
          <Text>请选择交易市场</Text>
        </Center>
      </Box>
    );
  }

  return (
    <Box bg="bg.card" borderRadius="md" p={4} height="430px">
      <Box ref={chartContainerRef} width="100%" height="400px" />
    </Box>
  );
};

export default TradingChart;