import { useRouter } from 'next/router'
import Minting from '../../components/Minting'
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount } from 'wagmi';
import Head from 'next/head';
import styles from '../../styles/Home.module.css';

export default () => {
  const router = useRouter()
  const account = useAccount()

  return (
    <div className={styles.container}>
      <Head>
        <title>QuibleMint</title>
        <meta
          content="Generated by @rainbow-me/create-rainbowkit"
          name="description"
        />
        <link href="/favicon.ico" rel="icon" />
      </Head>

      <main className={styles.main}>
        <ConnectButton />

        {account.isConnected && <Minting accountAddress={account.address!} tokenAddress={router.query.tokenAddress as unknown as string} />}
      </main>
    </div>
  );
}
