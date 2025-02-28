import React, { useMemo } from 'react';
import {
  Box,
  Flex,
  Text,
  Table,
  Thead,
  Tbody,
  Tr,
  Th,
  Td,
  Heading,
  Divider,
  Spinner,
  Center,
} from '@chakra-ui/react';
import { Market } from '@/types/market';
import { useOrderBook } from '@/hooks/useMarketData';
import { formatNumber } from '@/utils/format';

interface OrderBookProps {
  market: Market | null;
}

const OrderBook: React.FC<OrderBookProps> = ({ market }) => {
  const { data: orderBook, isLoading, error } = useOrderBook(market?.market_id);

  // 计算订单簿深度
  const { formattedAsks, formattedBids } = useMemo(() => {
    if (!orderBook) {
      return { formattedAsks: [], formattedBids: [] };
    }

    // 找到最高的累计数量，用于显示深度条
    let maxCumulativeAmount = 0;

    // 处理卖单（降序排列）
    const asks = [...orderBook.asks].sort((a, b) => a.price - b.price);
    let asksCumulative = 0;
    const formattedAsks = asks.map(order => {
      asksCumulative += order.size;
      maxCumulativeAmount = Math.max(maxCumulativeAmount, asksCumulative);
      return {
        ...order,
        total: asksCumulative,
      };
    });

    // 处理买单（降序排列）
    const bids = [...orderBook.bids].sort((a, b) => b.price - a.price);
    let bidsCumulative = 0;
    const formattedBids = bids.map(order => {
      bidsCumulative += order.size;
      maxCumulativeAmount = Math.max(maxCumulativeAmount, bidsCumulative);
      return {
        ...order,
        total: bidsCumulative,
      };
    });

    return {
      formattedAsks,
      formattedBids,
      maxCumulativeAmount,
    };
  }, [orderBook]);

  if (!market) {
    return (
      <Box bg="bg.card" borderRadius="md" p={4} height="100%">
        <Center>
          <Text>请选择交易市场</Text>
        </Center>
      </Box>
    );
  }

  return (
    <Box bg="bg.card" borderRadius="md" p={4} height="100%">
      <Heading size="sm" mb={4}>
        订单簿 {market.market_id}
      </Heading>

      {isLoading && (
        <Center py={10}>
          <Spinner />
        </Center>
      )}

      {error && (
        <Center py={10}>
          <Text color="red.500">加载订单簿失败</Text>
        </Center>
      )}

      {orderBook && (
        <Box>
          {/* 卖单 */}
          <Box maxH="200px" overflowY="auto" mb={2}>
            <Table variant="simple" size="sm">
              <Thead>
                <Tr>
                  <Th color="text.secondary" isNumeric>
                    价格
                  </Th>
                  <Th color="text.secondary" isNumeric>
                    数量
                  </Th>
                  <Th color="text.secondary" isNumeric>
                    总计
                  </Th>
                </Tr>
              </Thead>
              <Tbody>
                {formattedAsks.map((order) => (
                  <Tr key={order.order_id} position="relative">
                    {/* 深度条 */}
                    <Box
                      position="absolute"
                      top={0}
                      bottom={0}
                      right={0}
                      bg="rgba(239, 68, 68, 0.1)"
                      width={`${(order.total / (formattedAsks[formattedAsks.length - 1]?.total || 1)) * 100}%`}
                      zIndex={0}
                    />
                    <Td color="action.sell" isNumeric zIndex={1}>
                      {formatNumber(order.price, market.quote_decimals)}
                    </Td>
                    <Td isNumeric zIndex={1}>
                      {formatNumber(order.size, market.base_decimals)}
                    </Td>
                    <Td isNumeric zIndex={1}>
                      {formatNumber(order.total, market.base_decimals)}
                    </Td>
                  </Tr>
                ))}
              </Tbody>
            </Table>
          </Box>

          {/* 最新价格 */}
          <Box bg="bg.secondary" borderRadius="md" p={2} my={2}>
            <Flex justify="center">
              <Text fontSize="lg" fontWeight="bold" color="text.accent">
                {orderBook.bids[0] && formatNumber(orderBook.bids[0].price, market.quote_decimals)}
              </Text>
            </Flex>
          </Box>

          {/* 买单 */}
          <Box maxH="200px" overflowY="auto">
            <Table variant="simple" size="sm">
              <Thead>
                <Tr>
                  <Th color="text.secondary" isNumeric>
                    价格
                  </Th>
                  <Th color="text.secondary" isNumeric>
                    数量
                  </Th>
                  <Th color="text.secondary" isNumeric>
                    总计
                  </Th>
                </Tr>
              </Thead>
              <Tbody>
                {formattedBids.map((order) => (
                  <Tr key={order.order_id} position="relative">
                    {/* 深度条 */}
                    <Box
                      position="absolute"
                      top={0}
                      bottom={0}
                      right={0}
                      bg="rgba(16, 185, 129, 0.1)"
                      width={`${(order.total / (formattedBids[formattedBids.length - 1]?.total || 1)) * 100}%`}
                      zIndex={0}
                    />
                    <Td color="action.buy" isNumeric zIndex={1}>
                      {formatNumber(order.price, market.quote_decimals)}
                    </Td>
                    <Td isNumeric zIndex={1}>
                      {formatNumber(order.size, market.base_decimals)}
                    </Td>
                    <Td isNumeric zIndex={1}>
                      {formatNumber(order.total, market.base_decimals)}
                    </Td>
                  </Tr>
                ))}
              </Tbody>
            </Table>
          </Box>
        </Box>
      )}
    </Box>
  );
};

export default OrderBook;