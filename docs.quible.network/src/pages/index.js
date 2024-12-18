import Link from "@docusaurus/Link";
import Logo from "../../static/img/logo.svg";
import ExecutionEnvironment from "@docusaurus/ExecutionEnvironment";

const refreshDarkMode = () => {
  if (ExecutionEnvironment.canUseDOM) {
    const isDark =
      localStorage.theme === "dark" ||
      (!("theme" in localStorage) &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);

    if (isDark) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }
};

refreshDarkMode();

export default () => {
  setTimeout(() => refreshDarkMode(), 0);

  return (
    <div className="flex-grow relative h-full flex flex-col items-center justify-center text-xs dark:bg-black dark:text-white leading-4">
      <div className="font-mono whitespace-pre">
        {"                                  \n"}
        {"                                  \n"}
      </div>
      <Logo width="200" height="200" />

      <div className="font-mono whitespace-pre">
        {"                                  \n"}
        {"                                  \n"}
      </div>

      <div className="font-mono whitespace-pre">
        {"   ____         _  _      _       \n"}
        {"  /___ \\ _   _ (_)| |__  | |  ___ \n"}
        {" //  / /| | | || || '_ \\ | | / _ \\\n"}
        {"/ \\_/ / | |_| || || |_) || ||  __/\n"}
        {"\\___,_\\  \\__,_||_||_.__/ |_| \\___|\n"}
        {"                                  \n"}
        {"                                  \n"}
        {"                                  \n"}
      </div>

      <div className="font-mono whitespace-pre">
        {"                       docs                       \n"}
        {"                                                  \n"}
        {"                                                  \n"}
        {"+------------------------------------------------+\n"}
        {"|                                                |\n"}
        {"|  Quible Network                                |\n"}
        {"|  ==============                                |\n"}
        {"|                                                |\n"}
        {"|                                                |\n"}
        {"|  The decentralized security infrastructure     |\n"}
        {"|  marketplace, powered by zero-knowledge        |\n"}
        {"|  certificates, blockchain-based identity       |\n"}
        {"|  management, and more.                         |\n"}
        {"|                                                |\n"}
        {"|                                                |\n"}
        {"|  ------------------------------------          |\n"}
        {"|  | "}
        <Link
          className="!text-[blue] dark:!text-blue-300 font-bold underline"
          to="/intro"
        >
          ABOUT
        </Link>
        {" | "}
        <Link
          className="!text-[blue] dark:!text-blue-300 font-bold underline"
          to="/sdk"
        >
          USAGE
        </Link>
        {" | "}
        <a
          className="!text-[blue] dark:!text-blue-300 font-bold underline"
          href="https://ts.docsend.com/view/5xyq5v7burxnmkib"
          target="_blank"
        >
          WHITEPAPER
        </a>
        {" | "}
        <Link
          className="!text-[blue] dark:!text-blue-300 font-bold underline"
          to="/faq"
        >
          FAQ
        </Link>
        {" |          |\n"}
        {"|  ------------------------------------          |\n"}
        {"|                                                |\n"}
        {"+------------------------------------------------+\n"}
        {"                                                  \n"}
        {"                                                  \n"}
        {"                     "}
        <Link
          className="!text-[blue] dark:!text-blue-300 font-bold underline"
          to="/product"
        >
          WEBSITE
        </Link>
        {"                     \n"}
        {"                                                  \n"}
        {"                                                  \n"}
      </div>
    </div>
  );
};
