import { Background } from "@/components/blocks/landing/background";
import { DashedLine } from "@/components/blocks/landing/dashed-line";
import { Navbar } from "@/components/blocks/landing/navbar";
import { Footer } from "@/components/blocks/landing/footer";
import { AboutIntro } from "@/components/blocks/about/intro";
import { AboutTeam } from "@/components/blocks/about/team";

export default function AboutPage() {
  return (
    <>
      <Navbar />
      <Background>
        <AboutIntro />
        <DashedLine className="container max-w-3xl" />
        <AboutTeam />
      </Background>
      <Footer />
    </>
  );
}
