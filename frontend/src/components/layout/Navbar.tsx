import React from 'react';
import {
  Box,
  Flex,
  Text,
  IconButton,
  Button,
  Stack,
  Collapse,
  Icon,
  Link,
  Popover,
  PopoverTrigger,
  PopoverContent,
  useColorModeValue,
  useDisclosure,
  Image,
} from '@chakra-ui/react';
import {
  HamburgerIcon,
  CloseIcon,
  ChevronDownIcon,
  ChevronRightIcon,
} from '@chakra-ui/icons';
import NextLink from 'next/link';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import BuybackStats from '../common/BuybackStats';
import { useRouter } from 'next/router';

export default function Navbar() {
  const { isOpen, onToggle } = useDisclosure();
  const { connected, publicKey } = useWallet();
  const router = useRouter();

  const navItems = [
    { href: '/', label: 'Trade' },
    { href: '/pools', label: 'Pools' },
    { href: '/bridge', label: 'Bridge' },
    { href: '/governance', label: 'Governance' },
    { href: '/analytics', label: 'Analytics' },
    { href: '/docs', label: 'API Docs' },
  ];

  return (
    <Box as="nav" bg="bg.secondary" boxShadow="md">
      <Flex
        minH={'60px'}
        py={{ base: 2 }}
        px={{ base: 4, md: 6 }}
        align={'center'}
        justify="space-between"
        maxW="container.xl"
        mx="auto"
      >
        <Flex flex={{ base: 1 }} justify={{ base: 'start', md: 'start' }}>
          <NextLink href="/" passHref legacyBehavior>
            <Link>
              <Text
                textAlign="left"
                fontFamily={'heading'}
                color="text.accent"
                fontWeight="bold"
                fontSize="xl"
              >
                BitRS DEX
              </Text>
            </Link>
          </NextLink>

          <Flex display={{ base: 'none', md: 'flex' }} ml={10}>
            <DesktopNav />
          </Flex>
        </Flex>

        {/* 右侧钱包连接按钮等 */}
        <Stack
          flex={{ base: 1, md: 0 }}
          justify={'flex-end'}
          direction={'row'}
          spacing={6}
        >
          <Box
            className="wallet-adapter-button-trigger"
            sx={{
              '.wallet-adapter-button': {
                bg: 'action.button',
                color: 'white',
                fontSize: 'sm',
                fontWeight: '600',
                borderRadius: 'md',
                height: '38px',
                '&:hover': {
                  bg: 'action.buttonHover',
                },
              },
            }}
          >
            <WalletMultiButton />
          </Box>

          <Box display={{ base: 'none', lg: 'block' }} w="80">
            <BuybackStats />
          </Box>

          <IconButton
            display={{ base: 'flex', md: 'none' }}
            onClick={onToggle}
            icon={
              isOpen ? <CloseIcon w={3} h={3} /> : <HamburgerIcon w={5} h={5} />
            }
            variant={'ghost'}
            aria-label={'Toggle Navigation'}
          />
        </Stack>
      </Flex>

      <Collapse in={isOpen} animateOpacity>
        <MobileNav />
      </Collapse>
    </Box>
  );
}

const DesktopNav = () => {
  const linkColor = 'text.secondary';
  const linkHoverColor = 'text.primary';
  const popoverContentBgColor = 'bg.card';

  return (
    <Stack direction={'row'} spacing={4}>
      {NAV_ITEMS.map((navItem) => (
        <Box key={navItem.label}>
          <Popover trigger={'hover'} placement={'bottom-start'}>
            <PopoverTrigger>
              <Link
                p={2}
                href={navItem.href ?? '#'}
                fontSize={'sm'}
                fontWeight={500}
                color={linkColor}
                _hover={{
                  textDecoration: 'none',
                  color: linkHoverColor,
                }}
              >
                {navItem.label}
                {navItem.children && (
                  <Icon
                    as={ChevronDownIcon}
                    ml={1}
                    h={4}
                    w={4}
                    transition={'all .25s ease-in-out'}
                  />
                )}
              </Link>
            </PopoverTrigger>

            {navItem.children && (
              <PopoverContent
                border={0}
                boxShadow={'xl'}
                bg={popoverContentBgColor}
                p={4}
                rounded={'md'}
                minW={'sm'}
              >
                <Stack>
                  {navItem.children.map((child) => (
                    <DesktopSubNav key={child.label} {...child} />
                  ))}
                </Stack>
              </PopoverContent>
            )}
          </Popover>
        </Box>
      ))}
    </Stack>
  );
};

const DesktopSubNav = ({ label, href, subLabel }: NavItem) => {
  return (
    <Link
      href={href}
      role={'group'}
      display={'block'}
      p={2}
      rounded={'md'}
      _hover={{ bg: 'bg.hover' }}
    >
      <Stack direction={'row'} align={'center'}>
        <Box>
          <Text
            transition={'all .3s ease'}
            _groupHover={{ color: 'text.accent' }}
            fontWeight={500}
          >
            {label}
          </Text>
          <Text fontSize={'sm'} color={'text.secondary'}>
            {subLabel}
          </Text>
        </Box>
        <Flex
          transition={'all .3s ease'}
          transform={'translateX(-10px)'}
          opacity={0}
          _groupHover={{ opacity: '100%', transform: 'translateX(0)' }}
          justify={'flex-end'}
          align={'center'}
          flex={1}
        >
          <Icon color={'text.accent'} w={5} h={5} as={ChevronRightIcon} />
        </Flex>
      </Stack>
    </Link>
  );
};

const MobileNav = () => {
  return (
    <Stack
      bg={'bg.secondary'}
      p={4}
      display={{ md: 'none' }}
      borderBottom="1px"
      borderColor="gray.700"
    >
      {NAV_ITEMS.map((navItem) => (
        <MobileNavItem key={navItem.label} {...navItem} />
      ))}
    </Stack>
  );
};

const MobileNavItem = ({ label, children, href }: NavItem) => {
  const { isOpen, onToggle } = useDisclosure();

  return (
    <Stack spacing={4} onClick={children && onToggle}>
      <Flex
        py={2}
        as={Link}
        href={href ?? '#'}
        justify={'space-between'}
        align={'center'}
        _hover={{
          textDecoration: 'none',
        }}
      >
        <Text fontWeight={600} color={'text.primary'}>
          {label}
        </Text>
        {children && (
          <Icon
            as={ChevronDownIcon}
            transition={'all .25s ease-in-out'}
            transform={isOpen ? 'rotate(180deg)' : ''}
            w={6}
            h={6}
          />
        )}
      </Flex>

      <Collapse in={isOpen} animateOpacity style={{ marginTop: '0!important' }}>
        <Stack
          mt={2}
          pl={4}
          borderLeft={1}
          borderStyle={'solid'}
          borderColor={'gray.700'}
          align={'start'}
        >
          {children &&
            children.map((child) => (
              <Link key={child.label} py={2} href={child.href}>
                {child.label}
              </Link>
            ))}
        </Stack>
      </Collapse>
    </Stack>
  );
};

interface NavItem {
  label: string;
  subLabel?: string;
  children?: NavItem[];
  href?: string;
}

const NAV_ITEMS: NavItem[] = [
  {
    label: '交易',
    href: '/',
  },
  {
    label: '流动性池',
    href: '/pools',
  },
  {
    label: '资产',
    href: '/assets',
  },
  {
    label: '更多',
    children: [
      {
        label: '桥接',
        subLabel: '跨链资产转移',
        href: '/bridge',
      },
      {
        label: '文档',
        subLabel: 'API和开发者文档',
        href: '/docs',
      },
      {
        label: '分析',
        subLabel: '数据分析',
        href: '/analytics',
      },
    ],
  },
];