import { Background } from "@/components/blocks/landing/background";
import { FAQ } from "@/components/blocks/landing/faq";
import { Features } from "@/components/blocks/landing/features";
import { Footer } from "@/components/blocks/landing/footer";
import { Hero } from "@/components/blocks/landing/hero";
import { Navbar } from "@/components/blocks/landing/navbar";
import { ResourceAllocation } from "@/components/blocks/landing/resource-allocation";

export default function Home() {
  return (
    <>
      <Navbar />
      <Background className="via-muted to-muted/80">
        <Hero />
        <Features />
        <ResourceAllocation />
      </Background>
      <Background variant="bottom">
        <FAQ />
      </Background>
      <Footer />
    </>
  );
}
