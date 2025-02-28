import React from 'react';
import { Box, Container, Stack, Text, Link, useColorModeValue, Flex } from '@chakra-ui/react';

export default function Footer() {
  return (
    <Box bg="bg.secondary" color="text.secondary" borderTop="1px" borderColor="gray.700">
      <Container
        as={Stack}
        maxW="container.xl"
        py={4}
        direction={{ base: 'column', md: 'row' }}
        spacing={4}
        justify={{ base: 'center', md: 'space-between' }}
        align={{ base: 'center', md: 'center' }}
      >
        <Text>© 2023 BitRS DEX. 基于Solana的分布式交易所</Text>
        <Stack direction="row" spacing={6}>
          <Link href={'#'}>首页</Link>
          <Link href={'#'}>关于</Link>
          <Link href={'#'}>条款</Link>
          <Link href={'#'}>联系我们</Link>
        </Stack>
      </Container>
    </Box>
  );
}