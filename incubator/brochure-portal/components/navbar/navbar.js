import Image from "next/image";
import BNALogo from "../../public/bna-full-white-text-109×64.png";

export default function Navbar() {
  return (
    <header className="text-gray-600 body-font">
      <div className=" mx-auto flex flex-wrap p-1 flex-col md:flex-row items-center bg-pfb-deep-navy">
        <Image src={BNALogo} alt="BNA Logo with white text"></Image>
        <a className="flex title-font font-medium items-center text-gray-900 mb-1 md:mb-0">
          <span className="ml-3 text-4xl text-white">BNA Scorecards</span>
        </a>
        <nav className="md:ml-auto flex flex-wrap items-center text-base justify-center"></nav>
      </div>
    </header>
  );
}
