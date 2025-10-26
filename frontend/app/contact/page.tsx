import { Background } from "@/components/blocks/landing/background";
import { DashedLine } from "@/components/blocks/landing/dashed-line";
import { Footer } from "@/components/blocks/landing/footer";
import { Navbar } from "@/components/blocks/landing/navbar";
import { ContactCommunity } from "@/components/blocks/contact/community";
import { ContactSummary } from "@/components/blocks/contact/info";

export default function ContactPage() {
  return (
    <>
      <Navbar />
      <Background>
        <ContactSummary />
        <DashedLine className="container max-w-4xl" />
        <ContactCommunity />
      </Background>
      <Footer />
    </>
  );
}
