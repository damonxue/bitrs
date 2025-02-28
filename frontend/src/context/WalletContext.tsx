import React, { FC, ReactNode, useMemo, createContext, useContext, useState, useEffect } from 'react';
import { ConnectionProvider, WalletAdapterNetwork } from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork as Network } from '@solana/wallet-adapter-base';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { clusterApiUrl, PublicKey, Transaction, Connection } from '@solana/web3.js';
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
  TorusWalletAdapter,
} from '@solana/wallet-adapter-wallets';
import { WalletProvider as SolanaWalletProvider, useWallet as useSolanaWallet } from '@solana/wallet-adapter-react';
import { ethers } from 'ethers';
import { ERC20_ABI } from '@/constants/abis';

// 导入钱包适配器样式
import '@solana/wallet-adapter-react-ui/styles.css';

interface WalletProviderProps {
  children: ReactNode;
}

interface WalletContextType {
  wallet: any;
  connected: boolean;
  publicKey: PublicKey | null;
  signTransaction: (transaction: Transaction) => Promise<Transaction>;
  signAllTransactions: (transactions: Transaction[]) => Promise<Transaction[]>;
  disconnect: () => Promise<void>;
  isContractAccount: (address: string) => Promise<boolean>;
  validateTransaction: (transaction: Transaction) => Promise<boolean>;
  approveToken: (tokenAddress: string, amount: string, spender: string) => Promise<boolean>;
  getEthereumAssets: () => Promise<any[]>;
  getBSCAssets: () => Promise<any[]>;
  bridgeTokenApproval: (tokenAddress: string, amount: string) => Promise<boolean>;
}

const WalletContext = createContext<WalletContextType | undefined>(undefined);

const RESTRICTED_PROGRAMS = [
  // Add program IDs that should be restricted for contract accounts
];

export const WalletProvider: FC<WalletProviderProps> = ({ children }) => {
  const solanaWallet = useSolanaWallet();
  const [connection] = useState(new Connection(process.env.NEXT_PUBLIC_RPC_URL || ''));
  const [ethereumProvider, setEthereumProvider] = useState<ethers.providers.Web3Provider | null>(null);
  const [bscProvider, setBscProvider] = useState<ethers.providers.Web3Provider | null>(null);

  useEffect(() => {
    if (window.ethereum) {
      setEthereumProvider(new ethers.providers.Web3Provider(window.ethereum));
    }
    // Add BSC provider initialization if needed
  }, []);

  // 设置网络
  const network = (process.env.SOLANA_NETWORK as Network) || WalletAdapterNetwork.Devnet;

  // 使用自定义RPC URL或默认的集群API URL
  const endpoint = useMemo(() => {
    return process.env.SOLANA_RPC_URL || clusterApiUrl(network);
  }, [network]);

  // 初始化钱包适配器
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
      new TorusWalletAdapter(),
    ],
    []
  );

  const isContractAccount = async (address: string): Promise<boolean> => {
    try {
      const accountInfo = await connection.getAccountInfo(new PublicKey(address));
      return accountInfo?.executable || false;
    } catch (error) {
      console.error('Error checking account type:', error);
      return false;
    }
  };

  const validateTransaction = async (transaction: Transaction): Promise<boolean> => {
    try {
      // Check for suspicious instructions
      for (const ix of transaction.instructions) {
        // Prevent contract accounts from calling restricted programs
        if (RESTRICTED_PROGRAMS.includes(ix.programId.toString())) {
          const isContract = await isContractAccount(ix.keys[0].pubkey.toString());
          if (isContract) {
            console.error('Contract account attempting to access restricted program');
            return false;
          }
        }

        // Validate transaction size
        if (transaction.serialize().length > 1232) {
          console.error('Transaction size exceeds limit');
          return false;
        }
      }

      // Check for reasonable gas fees
      const fees = await connection.getFeeForMessage(transaction.compileMessage());
      if (fees.value > 1000000) { // 0.001 SOL
        console.error('Transaction fees too high');
        return false;
      }

      return true;
    } catch (error) {
      console.error('Error validating transaction:', error);
      return false;
    }
  };

  const signTransaction = async (transaction: Transaction): Promise<Transaction> => {
    if (!solanaWallet.signTransaction) {
      throw new Error('Wallet does not support transaction signing');
    }

    // Validate transaction before signing
    const isValid = await validateTransaction(transaction);
    if (!isValid) {
      throw new Error('Transaction validation failed');
    }

    return await solanaWallet.signTransaction(transaction);
  };

  const signAllTransactions = async (transactions: Transaction[]): Promise<Transaction[]> => {
    if (!solanaWallet.signAllTransactions) {
      throw new Error('Wallet does not support batch transaction signing');
    }

    // Validate all transactions before signing
    for (const tx of transactions) {
      const isValid = await validateTransaction(tx);
      if (!isValid) {
        throw new Error('Transaction validation failed');
      }
    }

    return await solanaWallet.signAllTransactions(transactions);
  };

  const approveToken = async (tokenAddress: string, amount: string, spender: string): Promise<boolean> => {
    try {
      if (!ethereumProvider) {
        throw new Error('Ethereum provider not available');
      }

      const signer = ethereumProvider.getSigner();
      const tokenContract = new ethers.Contract(tokenAddress, ERC20_ABI, signer);
      
      const tx = await tokenContract.approve(spender, amount);
      await tx.wait();
      return true;
    } catch (error) {
      console.error('Token approval failed:', error);
      return false;
    }
  };

  const getEthereumAssets = async (): Promise<any[]> => {
    if (!ethereumProvider || !solanaWallet.publicKey) return [];

    try {
      // Implementation to fetch Ethereum assets
      // This would typically involve querying a token list and checking balances
      return [];
    } catch (error) {
      console.error('Failed to fetch Ethereum assets:', error);
      return [];
    }
  };

  const getBSCAssets = async (): Promise<any[]> => {
    if (!bscProvider || !solanaWallet.publicKey) return [];

    try {
      // Implementation to fetch BSC assets
      return [];
    } catch (error) {
      console.error('Failed to fetch BSC assets:', error);
      return [];
    }
  };

  const bridgeTokenApproval = async (tokenAddress: string, amount: string): Promise<boolean> => {
    try {
      const bridgeAddress = process.env.NEXT_PUBLIC_BRIDGE_ADDRESS;
      if (!bridgeAddress) throw new Error('Bridge address not configured');

      return await approveToken(tokenAddress, amount, bridgeAddress);
    } catch (error) {
      console.error('Bridge approval failed:', error);
      return false;
    }
  };

  const value = {
    wallet: solanaWallet,
    connected: solanaWallet.connected,
    publicKey: solanaWallet.publicKey,
    signTransaction,
    signAllTransactions,
    disconnect: solanaWallet.disconnect,
    isContractAccount,
    validateTransaction,
    approveToken,
    getEthereumAssets,
    getBSCAssets,
    bridgeTokenApproval,
  };

  return (
    <ConnectionProvider endpoint={endpoint}>
      <SolanaWalletProvider wallets={wallets} autoConnect>
        <WalletContext.Provider value={value}>
          <WalletModalProvider>{children}</WalletModalProvider>
        </WalletContext.Provider>
      </SolanaWalletProvider>
    </ConnectionProvider>
  );
};

export const useWallet = () => {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
};

export default WalletProvider;